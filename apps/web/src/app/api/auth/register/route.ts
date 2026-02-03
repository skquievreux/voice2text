import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { kv } from '@vercel/kv';
import { LicenseManager } from '@/lib/license';
import { generateTokens } from '@/lib/auth';

export const runtime = 'edge';

const registerSchema = z.object({
    email: z.string().email(),
    licenseKey: z.string(),
    deviceId: z.string()
});

export async function POST(req: NextRequest) {
    try {
        const body = await req.json();
        const { email, licenseKey, deviceId } = registerSchema.parse(body);

        // 1. Validate License
        const licenseManager = new LicenseManager();
        const { valid, tier } = licenseManager.validateLicenseKey(licenseKey);

        if (!valid || !tier) {
            return NextResponse.json({ error: 'Invalid license key' }, { status: 400 });
        }

        // 2. Check if already activated
        const existingUserId = await kv.get(`license:${licenseKey}`);
        if (existingUserId) {
            return NextResponse.json({ error: 'License key already activated' }, { status: 400 });
        }

        // 3. Create User
        const userId = crypto.randomUUID();
        const user = {
            id: userId,
            email,
            tier,
            licenseKey,
            devices: [deviceId],
            createdAt: Date.now()
        };

        await kv.set(`user:${userId}`, user);
        await kv.set(`license:${licenseKey}`, userId);
        await kv.set(`email:${email}`, userId);

        const { accessToken, refreshToken } = await generateTokens(userId, tier);

        return NextResponse.json({
            accessToken,
            refreshToken,
            user: { id: userId, email, tier }
        });

    } catch (error) {
        if (error instanceof z.ZodError) {
            return NextResponse.json({ error: 'Invalid input', details: error.issues }, { status: 400 });
        }
        console.error('[AUTH_REGISTER_ERROR]', error);
        return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
    }
}
