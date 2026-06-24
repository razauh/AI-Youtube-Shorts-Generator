export interface ProtectedStoreAdapter {
  read(path: string): Promise<string | null>;
  write(path: string, value: string): Promise<void>;
  remove(path: string): Promise<void>;
}

export interface ProtectedStore {
  save(key: string, value: unknown): Promise<void>;
  load<T>(key: string): Promise<T | null>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
}

type EncryptedPayload = {
  version: 1;
  salt: string;
  iv: string;
  ciphertext: string;
};

function bytesToBase64(bytes: Uint8Array): string {
  if (typeof Buffer !== 'undefined') {
    return Buffer.from(bytes).toString('base64');
  }
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  if (typeof Buffer !== 'undefined') {
    return new Uint8Array(Buffer.from(value, 'base64'));
  }
  const binary = atob(value);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    out[i] = binary.charCodeAt(i);
  }
  return out;
}

function randomBytes(length: number): Uint8Array {
  const out = new Uint8Array(length);
  crypto.getRandomValues(out);
  return out;
}

async function deriveKey(secret: string, salt: Uint8Array): Promise<CryptoKey> {
  const baseKey = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    'PBKDF2',
    false,
    ['deriveKey']
  );
  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations: 100_000,
      hash: 'SHA-256',
    },
    baseKey,
    {
      name: 'AES-GCM',
      length: 256,
    },
    false,
    ['encrypt', 'decrypt']
  );
}

async function encodePayload(secret: string, value: unknown): Promise<string> {
  const salt = randomBytes(16);
  const iv = randomBytes(12);
  const key = await deriveKey(secret, salt);
  const ciphertext = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv,
    },
    key,
    new TextEncoder().encode(JSON.stringify(value))
  );
  const payload: EncryptedPayload = {
    version: 1,
    salt: bytesToBase64(salt),
    iv: bytesToBase64(iv),
    ciphertext: bytesToBase64(new Uint8Array(ciphertext)),
  };
  return JSON.stringify(payload);
}

async function decodePayload<T>(secret: string, payload: string): Promise<T> {
  const decoded = JSON.parse(payload) as Partial<EncryptedPayload>;
  if (
    decoded.version !== 1 ||
    typeof decoded.salt !== 'string' ||
    typeof decoded.iv !== 'string' ||
    typeof decoded.ciphertext !== 'string'
  ) {
    throw new Error('invalid encrypted payload');
  }
  const salt = base64ToBytes(decoded.salt);
  const iv = base64ToBytes(decoded.iv);
  const ciphertext = base64ToBytes(decoded.ciphertext);
  const key = await deriveKey(secret, salt);
  const plaintext = await crypto.subtle.decrypt(
    {
      name: 'AES-GCM',
      iv,
    },
    key,
    ciphertext
  );
  return JSON.parse(new TextDecoder().decode(plaintext)) as T;
}

export class EncryptedProtectedStore implements ProtectedStore {
  constructor(
    private readonly adapter: ProtectedStoreAdapter,
    private readonly secret: string,
    private readonly basePath = 'protected'
  ) {}

  private pathFor(key: string): string {
    return `${this.basePath}/${key}.enc`;
  }

  async save(key: string, value: unknown): Promise<void> {
    const path = this.pathFor(key);
    await this.adapter.write(path, await encodePayload(this.secret, value));
  }

  async load<T>(key: string): Promise<T | null> {
    const path = this.pathFor(key);
    const raw = await this.adapter.read(path);
    if (!raw) return null;
    try {
      return await decodePayload<T>(this.secret, raw);
    } catch {
      await this.adapter.remove(path);
      return null;
    }
  }

  async delete(key: string): Promise<void> {
    await this.adapter.remove(this.pathFor(key));
  }

  async clear(): Promise<void> {
    // callers may delete known keys explicitly
  }
}
