import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, cleanup } from '@testing-library/svelte';
import Page from '../routes/+page.svelte';
import { createRunState } from '../lib/stores/runState';

const runGenerateAndStream = vi.fn();
const pickLocalVideoFile = vi.fn();

vi.mock('../lib/api/tauriClient', () => ({
  runGenerateAndStream: (...args: unknown[]) => runGenerateAndStream(...args),
  pickLocalVideoFile: (...args: unknown[]) => pickLocalVideoFile(...args)
}));

describe('ui flow parity', () => {
  beforeEach(() => {
    runGenerateAndStream.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it('default form values parity', () => {
    render(Page);

    expect((screen.getByLabelText('YouTube video URL') as HTMLInputElement).value).toBe('');
    expect((screen.getByLabelText('Mode') as HTMLSelectElement).value).toBe('api');
    expect((screen.getByLabelText('Num clips') as HTMLInputElement).value).toBe('3');
    expect((screen.getByLabelText('Aspect ratio') as HTMLSelectElement).value).toBe('9:16');
    expect((screen.getByLabelText('Resolution') as HTMLSelectElement).value).toBe('720');
    expect((screen.getByLabelText('Output JSON path') as HTMLInputElement).value).toBe('');
  });

  it('submit sends correct payload', async () => {
    runGenerateAndStream.mockResolvedValue({
      ok: true,
      result: { mode: 'api', source_video_url: 'x', transcript: { duration: 1, segments: [] }, highlights: [], shorts: [] }
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.change(screen.getByLabelText('Mode'), { target: { value: 'local' } });
    await fireEvent.input(screen.getByLabelText('Num clips'), { target: { value: '5' } });
    await fireEvent.input(screen.getByLabelText('Resolution'), { target: { value: '1080' } });
    await fireEvent.change(screen.getByLabelText('Aspect ratio'), { target: { value: '1:1' } });
    await fireEvent.input(screen.getByLabelText('Output JSON path'), { target: { value: 'result.json' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(runGenerateAndStream).toHaveBeenCalledTimes(1);
    const req = runGenerateAndStream.mock.calls[0][0];
    expect(req).toEqual({
      youtube_url: 'https://youtube.com/watch?v=abc',
      mode: 'local',
      num_clips: 5,
      aspect_ratio: '1:1',
      download_format: '1080',
      output_json: 'result.json'
    });
  });

  it('event updates state lifecycle and progress', async () => {
    const state = createRunState();
    state.start();
    state.onProgress({ event: 'progress', stage: 'download:start', progress: 0.1 });
    state.onProgress({ event: 'progress', stage: 'clip:end', progress: 1.0 });

    let latest: any;
    const unsub = state.subscribe((v) => {
      latest = v;
    });

    expect(latest.lifecycle).toBe('running');
    expect(latest.progress.stage).toBe('clip:end');
    expect(latest.progress.value).toBe(1);
    unsub();
  });

  it('render short entries and failures', async () => {
    runGenerateAndStream.mockResolvedValue({
      ok: true,
      result: {
        mode: 'api',
        source_video_url: 'https://cdn.example.com/video.mp4',
        transcript: { duration: 1, segments: [] },
        highlights: [{ title: 'H1', start_time: 1, end_time: 2, score: 90, hook_sentence: 'hook', virality_reason: 'reason' }],
        shorts: [
          { title: 'Hit', start_time: 1, end_time: 2, score: 90, hook_sentence: 'hook', virality_reason: 'reason', clip_url: 'https://cdn.example.com/s1.mp4' },
          { title: 'Miss', start_time: 3, end_time: 4, score: 60, hook_sentence: 'hook2', virality_reason: 'reason2', clip_url: null, error: 'render failed' }
        ]
      }
    });

    render(Page);
    await fireEvent.input(screen.getByLabelText('YouTube video URL'), { target: { value: 'https://youtube.com/watch?v=abc' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(await screen.findByText('Hit')).toBeTruthy();
    expect(await screen.findByText('Miss')).toBeTruthy();
    expect(await screen.findByText(/FAILED \(render failed\)/)).toBeTruthy();
  });
});
