import { writable } from 'svelte/store';
import type { ErrorEnvelope, PipelineSuccess, ProgressEvent } from '../contracts';

export type RunLifecycle = 'idle' | 'running' | 'success' | 'error';

export interface RunStateShape {
  lifecycle: RunLifecycle;
  runId: string | null;
  progress: {
    stage: string;
    value: number;
    message: string;
  };
  result: PipelineSuccess | null;
  error: ErrorEnvelope | null;
}

const initialState: RunStateShape = {
  lifecycle: 'idle',
  runId: null,
  progress: {
    stage: 'idle',
    value: 0,
    message: ''
  },
  result: null,
  error: null
};

export function createRunState() {
  const { subscribe, set, update } = writable<RunStateShape>(initialState);

  return {
    subscribe,
    reset: () => set(initialState),
    start: (runId: string) =>
      update((state) => ({
        ...state,
        lifecycle: 'running',
        runId,
        progress: { stage: 'start', value: 0, message: '' },
        result: null,
        error: null
      })),
    onProgress: (event: ProgressEvent) =>
      update((state) => {
        if (state.runId && event.run_id && state.runId !== event.run_id) {
          return state;
        }
        const nextRunId = state.runId ?? event.run_id ?? null;
        return {
          ...state,
          runId: nextRunId,
          lifecycle: state.lifecycle === 'idle' ? 'running' : state.lifecycle,
          progress: {
            stage: event.stage,
            value: event.progress,
            message: event.message ?? ''
          }
        };
      }),
    onSuccess: (result: PipelineSuccess) =>
      update((state) => ({
        ...state,
        lifecycle: 'success',
        progress: { ...state.progress, value: 1 },
        result,
        error: null
      })),
    onError: (error: ErrorEnvelope) =>
      update((state) => ({
        ...state,
        lifecycle: 'error',
        error,
        result: null
      }))
  };
}

export const runState = createRunState();
