export interface SecureStore {
  save(key: string, value: string): Promise<void>;
  load(key: string): Promise<string | null>;
  delete(key: string): Promise<void>;
  exists(key: string): Promise<boolean>;
}

export interface SecureStoreInvoker {
  save(key: string, value: string): Promise<void>;
  load(key: string): Promise<string | null>;
  delete(key: string): Promise<void>;
  exists(key: string): Promise<boolean>;
}

export class InMemorySecureStore implements SecureStore {
  private readonly map = new Map<string, string>();

  async save(key: string, value: string): Promise<void> {
    this.map.set(key, value);
  }

  async load(key: string): Promise<string | null> {
    return this.map.get(key) ?? null;
  }

  async delete(key: string): Promise<void> {
    this.map.delete(key);
  }

  async exists(key: string): Promise<boolean> {
    return this.map.has(key);
  }
}

export class TauriSecureStore implements SecureStore {
  constructor(private readonly invoker: SecureStoreInvoker) {}

  save(key: string, value: string): Promise<void> {
    return this.invoker.save(key, value);
  }

  load(key: string): Promise<string | null> {
    return this.invoker.load(key);
  }

  delete(key: string): Promise<void> {
    return this.invoker.delete(key);
  }

  exists(key: string): Promise<boolean> {
    return this.invoker.exists(key);
  }
}

export interface SimpleProtectedStringStoreAdapter {
  save(key: string, value: string): Promise<void>;
  load(key: string): Promise<string | null>;
  delete(key: string): Promise<void>;
}

export class ProtectedStringSecureStore implements SecureStore {
  constructor(private readonly adapter: SimpleProtectedStringStoreAdapter) {}

  save(key: string, value: string): Promise<void> {
    return this.adapter.save(key, value);
  }

  load(key: string): Promise<string | null> {
    return this.adapter.load(key);
  }

  delete(key: string): Promise<void> {
    return this.adapter.delete(key);
  }

  async exists(key: string): Promise<boolean> {
    return (await this.load(key)) !== null;
  }
}

export class FallbackSecureStore implements SecureStore {
  constructor(
    private readonly primary: SecureStore,
    private readonly fallback: SecureStore,
    private readonly onFallback?: (code: 'STOR_KEYCHAIN_UNAVAILABLE') => void
  ) {}

  private async withFallback<T>(fn: (store: SecureStore) => Promise<T>): Promise<T> {
    try {
      return await fn(this.primary);
    } catch {
      this.onFallback?.('STOR_KEYCHAIN_UNAVAILABLE');
      return fn(this.fallback);
    }
  }

  save(key: string, value: string): Promise<void> {
    return this.withFallback((s) => s.save(key, value));
  }
  load(key: string): Promise<string | null> {
    return this.withFallback((s) => s.load(key));
  }
  delete(key: string): Promise<void> {
    return this.withFallback((s) => s.delete(key));
  }
  exists(key: string): Promise<boolean> {
    return this.withFallback((s) => s.exists(key));
  }
}
