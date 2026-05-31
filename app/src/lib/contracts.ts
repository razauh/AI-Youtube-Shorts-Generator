export interface TranscriptSegment {
  start: number;
  end: number;
  text: string;
  [key: string]: unknown;
}

export interface Transcript {
  duration: number;
  segments: TranscriptSegment[];
  [key: string]: unknown;
}

export interface Highlight {
  title: string;
  start_time: number;
  end_time: number;
  score: number;
  hook_sentence: string;
  virality_reason: string;
  [key: string]: unknown;
}

export interface ShortClip {
  title: string;
  start_time: number;
  end_time: number;
  score: number;
  hook_sentence: string;
  virality_reason: string;
  clip_url: string | null;
  error?: string | null;
  [key: string]: unknown;
}

export interface PipelineSuccess {
  mode: string;
  source_video_url: string;
  transcript: Transcript;
  highlights: Highlight[];
  shorts: ShortClip[];
  [key: string]: unknown;
}

export interface ErrorEnvelope {
  mode?: string | null;
  source_video_url?: string | null;
  error: string;
  details?: unknown;
}

export interface ProgressEvent {
  event: string;
  run_id?: string;
  stage: string;
  progress: number;
  message?: string;
  mode?: string;
  source_video_url?: string;
  [key: string]: unknown;
}
