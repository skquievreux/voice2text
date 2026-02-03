import crypto from 'crypto';

export class LicenseManager {
    private encryptionKey: Buffer;

    constructor() {
        const key = process.env.LICENSE_ENCRYPTION_KEY;
        if (!key || key.length !== 64) {
            throw new Error('LICENSE_ENCRYPTION_KEY must be a 64-char hex string (32 bytes)');
        }
        this.encryptionKey = Buffer.from(key, 'hex');
    }

    // Format: VT-XXXX-XXXX-XXXX-XXXX
    generateLicenseKey(tier: 'free' | 'basic' | 'pro'): string {
        const data = {
            tier,
            created: Date.now(),
            random: crypto.randomBytes(8).toString('hex')
        };

        const json = JSON.stringify(data);
        const iv = crypto.randomBytes(12); // GCM standard IV size
        const cipher = crypto.createCipheriv('aes-256-gcm', this.encryptionKey, iv);

        const encrypted = Buffer.concat([cipher.update(json, 'utf8'), cipher.final()]);
        const tag = cipher.getAuthTag();

        // Pack: IV(12) + Tag(16) + Encrypted
        const packed = Buffer.concat([iv, tag, encrypted]);
        const hex = packed.toString('hex').toUpperCase();

        // Format as XXXX-XXXX-XXXX-XXXX...
        return `VT-${hex.slice(0, 4)}-${hex.slice(4, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}`;
    }

    validateLicenseKey(key: string): { valid: boolean; tier?: 'free' | 'basic' | 'pro' } {
        try {
            const hex = key.replace('VT-', '').replace(/-/g, '');
            const packed = Buffer.from(hex, 'hex');

            const iv = packed.subarray(0, 12);
            const tag = packed.subarray(12, 28);
            const encrypted = packed.subarray(28);

            const decipher = crypto.createDecipheriv('aes-256-gcm', this.encryptionKey, iv);
            decipher.setAuthTag(tag);

            const decrypted = Buffer.concat([decipher.update(encrypted), decipher.final()]);
            const data = JSON.parse(decrypted.toString('utf8'));

            return { valid: true, tier: data.tier };
        } catch (e) {
            console.error('License validation failed:', e);
            return { valid: false };
        }
    }
}
