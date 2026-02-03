// This would normally use @vercel/kv, using a mock for now
// apps/web/src/lib/rate-limiter.ts

export async function checkRateLimit(userId: string, tier: 'free' | 'pro') {
    // In a real Vercel environment:
    // const key = `ratelimit:${userId}:${new Date().getUTCDate()}`;
    // const count = await kv.incr(key);
    // if (count === 1) await kv.expire(key, 86400);

    // Fake logic for demo
    const limit = tier === 'pro' ? 10000 : 100;
    const current = 10; // Mocked

    return {
        success: current < limit,
        remaining: limit - current,
    };
}
