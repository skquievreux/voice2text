import { SignJWT, jwtVerify } from 'jose';

const JWT_SECRET = new TextEncoder().encode(process.env.JWT_SECRET || 'fallback_secret_min_32_chars_long');
const REFRESH_SECRET = new TextEncoder().encode(process.env.JWT_REFRESH_SECRET || 'fallback_refresh_secret_min_32_chars_long');

export async function generateTokens(userId: string, tier: string) {
    const accessToken = await new SignJWT({ tier })
        .setProtectedHeader({ alg: 'HS256' })
        .setSubject(userId)
        .setIssuedAt()
        .setExpirationTime('7d')
        .sign(JWT_SECRET);

    const refreshToken = await new SignJWT({})
        .setProtectedHeader({ alg: 'HS256' })
        .setSubject(userId)
        .setIssuedAt()
        .setExpirationTime('30d')
        .sign(REFRESH_SECRET);

    return { accessToken, refreshToken };
}

export async function verifyAccessToken(token: string) {
    try {
        const { payload } = await jwtVerify(token, JWT_SECRET);
        return payload;
    } catch {
        return null;
    }
}

export async function verifyRefreshToken(token: string) {
    try {
        const { payload } = await jwtVerify(token, REFRESH_SECRET);
        return payload;
    } catch {
        return null;
    }
}
