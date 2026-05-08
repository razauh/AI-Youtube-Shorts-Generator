import type { ErrorEnvelope, PipelineSuccess, ProgressEvent } from '../contracts';

export interface GenerateRequest {
  youtube_url: string;
  mode: 'api' | 'local';
  num_clips: number;
  aspect_ratio: string;
  download_format: string;
  language?: string;
  output_json?: string;
}

export type GenerateEnvelope =
  | { ok: true; result: PipelineSuccess }
  | { ok: false; error: ErrorEnvelope };
type GenerateWithEventsResponse = {
  envelope: GenerateEnvelope;
  events: ProgressEvent[];
};

interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}
type TauriEventApi = {
  listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void>;
};

let corePromise: Promise<TauriCore> | null = null;
let eventPromise: Promise<TauriEventApi> | null = null;

async function getCore(): Promise<TauriCore> {
  if (!corePromise) {
    corePromise = import('@tauri-apps/api/core') as Promise<TauriCore>;
  }
  return corePromise;
}

async function getEvents(): Promise<TauriEventApi> {
  if (!eventPromise) {
    eventPromise = import('@tauri-apps/api/event') as Promise<TauriEventApi>;
  }
  return eventPromise;
}

export async function runGenerateAndStream(
  request: GenerateRequest,
  onProgress: (event: ProgressEvent) => void
): Promise<GenerateEnvelope> {
  const eventsApi = await getEvents();
  const unlisten = await eventsApi.listen<ProgressEvent>('generate-progress', (event) => {
    onProgress(event.payload);
  });

  const { invoke } = await getCore();
  try {
    const envelope = await invoke<GenerateEnvelope>('generate_shorts_stream', {
      args: {
        request: {
          youtube_url: request.youtube_url,
          mode: request.mode,
          num_clips: request.num_clips,
          aspect_ratio: request.aspect_ratio,
          download_format: request.download_format,
          language: request.language && request.language.length > 0 ? request.language : null,
          output_json: request.output_json && request.output_json.length > 0 ? request.output_json : null
        }
      }
    });
    return envelope;
  } finally {
    unlisten();
  }
}
