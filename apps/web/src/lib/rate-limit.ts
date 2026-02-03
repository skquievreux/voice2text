import { Ratelimit } from '@upstash/ratelimit';
import { kv } from '@vercel/kv';

const RATE_LIMITS = {
    free: { requests: 10, window: '1 h' as const },
    basic: { requests: 100, window: '1 h' as const },
    pro: { requests: 1000, window: '1 h' as const },
};

export function getRateLimiter(tier: string) {
    const config = RATE_LIMITS[tier as keyof typeof RATE_LIMITS] || RATE_LIMITS.free;

    return new Ratelimit({
        redis: kv,
        limiter: Ratelimit.slidingWindow(config.requests, config.window),
        prefix: `ratelimit:${tier}`,
        analytics: true,
    });
}
