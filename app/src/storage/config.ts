export interface ConfigStoreAdapter {
  read(path: string): Promise<string | null>;
  write(path: string, value: string): Promise<void>;
}

export class JsonConfigStore {
  private cache: Record<string, string> | null = null;

  constructor(
    private readonly adapter: ConfigStoreAdapter,
    private readonly path = 'config.json',
    private readonly defaults: Record<string, string> = {}
  ) {}

  private async loadAll(): Promise<Record<string, string>> {
    if (this.cache) return this.cache;
    const raw = await this.adapter.read(this.path);
    if (!raw) {
      this.cache = { ...this.defaults };
      await this.adapter.write(this.path, JSON.stringify(this.cache));
      return this.cache;
    }
    try {
      this.cache = JSON.parse(raw) as Record<string, string>;
      return this.cache;
    } catch {
      this.cache = { ...this.defaults };
      await this.adapter.write(this.path, JSON.stringify(this.cache));
      return this.cache;
    }
  }

  async get(key: string): Promise<string | null> {
    const all = await this.loadAll();
    return all[key] ?? null;
  }

  async set(key: string, value: string): Promise<void> {
    const all = await this.loadAll();
    all[key] = value;
    await this.adapter.write(this.path, JSON.stringify(all));
  }

  async getAll(): Promise<Record<string, string>> {
    const all = await this.loadAll();
    return { ...all };
  }

  async reset(): Promise<void> {
    this.cache = { ...this.defaults };
    await this.adapter.write(this.path, JSON.stringify(this.cache));
  }
}
