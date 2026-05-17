export interface RetryPolicy {
  maxAttempts: number;
  backoffBaseMs: number;
  backoffMaxMs: number;
  retryableStatusCodes: number[];
}

export interface NetworkClientConfig {
  fetchImpl?: typeof fetch;
  timeoutMs?: number;
  retryPolicy?: RetryPolicy;
  sleep?: (ms: number) => Promise<void>;
}

export interface NetworkRequestConfig {
  method: string;
  url: string;
  headers?: Record<string, string>;
  body?: string;
  timeoutMs?: number;
  retryPolicy?: RetryPolicy;
  dedupeKey?: string;
}

export type NetworkFailureKind =
  | 'timeout'
  | 'dns_failure'
  | 'connection_refused'
  | 'ssl_error'
  | 'offline'
  | 'unknown';

export interface NetworkFailure {
  kind: NetworkFailureKind;
  code: `NET_${string}`;
  retryable: boolean;
}

const DEFAULT_RETRY: RetryPolicy = {
  maxAttempts: 3,
  backoffBaseMs: 1000,
  backoffMaxMs: 30000,
  retryableStatusCodes: [429, 502, 503, 504],
};

function backoffMs(attempt: number, policy: RetryPolicy): number {
  const raw = policy.backoffBaseMs * 2 ** Math.max(0, attempt - 1);
  return Math.min(raw, policy.backoffMaxMs);
}

export function classifyNetworkFailure(error: unknown): NetworkFailure {
  const name = error instanceof Error ? error.name : '';
  const message =
    error instanceof Error ? error.message : typeof error === 'string' ? error : String(error ?? '');
  const normalized = `${name} ${message}`.toLowerCase();

  if (name === 'AbortError' || normalized.includes('timeout') || normalized.includes('timed out')) {
    return { kind: 'timeout', code: 'NET_TIMEOUT', retryable: true };
  }
  if (normalized.includes('dns') || normalized.includes('enotfound') || normalized.includes('name not resolved')) {
    return { kind: 'dns_failure', code: 'NET_DNS_FAILURE', retryable: true };
  }
  if (
    normalized.includes('refused') ||
    normalized.includes('econnrefused') ||
    normalized.includes('connection refused')
  ) {
    return { kind: 'connection_refused', code: 'NET_CONNECTION_REFUSED', retryable: true };
  }
  if (
    normalized.includes('ssl') ||
    normalized.includes('tls') ||
    normalized.includes('certificate') ||
    normalized.includes('self signed')
  ) {
    return { kind: 'ssl_error', code: 'NET_SSL_ERROR', retryable: false };
  }
  if (normalized.includes('offline') || normalized.includes('failed to fetch') || normalized.includes('networkerror')) {
    return { kind: 'offline', code: 'NET_OFFLINE', retryable: true };
  }

  return { kind: 'unknown', code: 'NET_UNKNOWN', retryable: true };
}

export class NetworkClient {
  private readonly fetchImpl: typeof fetch;
  private readonly timeoutMs: number;
  private readonly retryPolicy: RetryPolicy;
  private readonly sleep: (ms: number) => Promise<void>;
  private online = true;
  private readonly onlineListeners = new Set<(online: boolean) => void>();
  private readonly inflight = new Map<string, Promise<Response>>();

  constructor(config: NetworkClientConfig = {}) {
    this.fetchImpl = config.fetchImpl ?? fetch;
    this.timeoutMs = config.timeoutMs ?? 10_000;
    this.retryPolicy = config.retryPolicy ?? DEFAULT_RETRY;
    this.sleep = config.sleep ?? ((ms) => new Promise((r) => setTimeout(r, ms)));
  }

  isOnline(): boolean {
    return this.online;
  }

  onConnectivityChange(callback: (online: boolean) => void): () => void {
    this.onlineListeners.add(callback);
    return () => this.onlineListeners.delete(callback);
  }

  private setOnline(next: boolean): void {
    if (this.online === next) return;
    this.online = next;
    for (const listener of this.onlineListeners) {
      listener(next);
    }
  }

  private dedupeKey(config: NetworkRequestConfig): string | null {
    if (config.dedupeKey) return config.dedupeKey;
    const m = config.method.toUpperCase();
    const idempotent = m === 'GET' || m === 'HEAD';
    if (!idempotent) return null;
    return `${m}:${config.url}:${config.body ?? ''}`;
  }

  async request(config: NetworkRequestConfig): Promise<Response> {
    const key = this.dedupeKey(config);
    if (key && this.inflight.has(key)) {
      return this.inflight.get(key)!;
    }
    const promise = this.requestInternal(config).finally(() => {
      if (key) this.inflight.delete(key);
    });
    if (key) this.inflight.set(key, promise);
    return promise;
  }

  private async requestInternal(config: NetworkRequestConfig): Promise<Response> {
    let lastErr: unknown;
    const policy = config.retryPolicy ?? this.retryPolicy;
    const timeoutMs = config.timeoutMs ?? this.timeoutMs;
    for (let attempt = 1; attempt <= policy.maxAttempts; attempt++) {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), timeoutMs);
      try {
        const response = await this.fetchImpl(config.url, {
          method: config.method,
          headers: config.headers,
          body: config.body,
          signal: controller.signal,
        });
        clearTimeout(timer);
        this.setOnline(true);
        if (
          policy.retryableStatusCodes.includes(response.status) &&
          attempt < policy.maxAttempts
        ) {
          await this.sleep(backoffMs(attempt, policy));
          continue;
        }
        return response;
      } catch (err) {
        clearTimeout(timer);
        lastErr = err;
        this.setOnline(false);
        if (attempt < policy.maxAttempts) {
          await this.sleep(backoffMs(attempt, policy));
          continue;
        }
      }
    }
    throw lastErr ?? new Error('network request failed');
  }
}

export function createNetworkClient(config: NetworkClientConfig = {}): NetworkClient {
  return new NetworkClient(config);
}
