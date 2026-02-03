import { NextRequest, NextResponse } from 'next/server';
import { verifyAccessToken } from '@/lib/auth';
import { getRateLimiter } from '@/lib/rate-limit';
import { kv } from '@vercel/kv';

export const runtime = 'edge';
export const dynamic = 'force-dynamic';

export async function POST(req: NextRequest) {
  try {
    // 1. Auth check
    const authHeader = req.headers.get('authorization');
    const token = authHeader?.replace('Bearer ', '');

    if (!token) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const payload = await verifyAccessToken(token);
    if (!payload || !payload.sub) {
      return NextResponse.json({ error: 'Invalid or expired token' }, { status: 401 });
    }

    const userId = payload.sub as string;
    const tier = (payload.tier as string) || 'free';

    // 2. Rate Limiting
    const rateLimiter = getRateLimiter(tier);
    const { success, remaining, reset } = await rateLimiter.limit(userId);

    if (!success) {
      return NextResponse.json(
        { error: 'Rate limit exceeded', resetAt: reset },
        {
          status: 429,
          headers: {
            'X-RateLimit-Remaining': remaining.toString(),
            'X-RateLimit-Reset': reset.toString()
          }
        }
      );
    }

    // 3. Transcription Logic (Deepgram Proxy)
    const api_key = process.env.DEEPGRAM_API_KEY;
    if (!api_key) {
      return NextResponse.json({ error: 'Configuration error' }, { status: 500 });
    }

    const formData = await req.formData();
    const audio = formData.get('audio') as File;

    if (!audio) {
      return NextResponse.json({ error: 'No audio provided' }, { status: 400 });
    }

    const response = await fetch(
      'https://api.deepgram.com/v1/listen?model=nova-2&language=de&smart_format=true',
      {
        method: 'POST',
        headers: {
          'Authorization': `Token ${api_key}`,
          'Content-Type': audio.type,
        },
        body: await audio.arrayBuffer(),
      }
    );

    const result = await response.json();
    const transcript = result.results?.channels[0]?.alternatives[0]?.transcript || "";
    const duration = result.metadata?.duration || 0;

    // 4. Usage Tracking
    const now = new Date();
    const monthKey = `usage:${userId}:${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
    await kv.hincrby(monthKey, 'minutes', Math.ceil(duration / 60));

    return NextResponse.json({
      text: transcript,
      duration,
      usage: {
        remaining,
        resetAt: reset
      }
    });

  } catch (error) {
    console.error('[TRANSCRIBE_API_ERROR]', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
