// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createApp, nextTick, type App } from 'vue';
import JsonTree from './JsonTree.vue';

vi.mock('vue-sonner', () => ({ toast: { success: vi.fn() } }));

let app: App;
let container: HTMLDivElement;
let copied: string;

beforeEach(() => {
  copied = '';
  container = document.createElement('div');
  document.body.appendChild(container);
  vi.stubGlobal('isSecureContext', true);
  vi.spyOn(navigator.clipboard, 'writeText').mockImplementation(async (text) => { copied = text; });
});

afterEach(() => {
  app?.unmount();
  container.remove();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function mount(data: unknown, name: string | number | null = null): HTMLButtonElement[] {
  app = createApp(JsonTree, { data, name });
  app.mount(container);
  return Array.from(container.querySelectorAll('button'));
}

async function click(button: HTMLButtonElement | undefined): Promise<void> {
  expect(button).toBeDefined();
  button!.click();
  await nextTick();
}

describe('JSON response copying', () => {
  it.each([
    ['fetchedAt', '2026-10-24T10:08:00.9967253+08:00', '"fetchedAt": "2026-10-24T10:08:00.9967253+08:00"'],
    ['rate', 6.674, '"rate": 6.674'],
    ['enabled', false, '"enabled": false'],
    ['count', 0, '"count": 0'],
    ['missing', null, '"missing": null'],
    ['', '', '"": ""'],
    ['a"b', 'line\n"quoted"', '"a\\"b": "line\\n\\"quoted\\""'],
  ])('copies both key and value for %s', async (name, data, expected) => {
    const controls = mount(data, name as string);
    await click(controls.find((button) => button.hasAttribute('aria-label')));
    expect(copied).toBe(expected);
  });

  it('keeps clicking the key separate from copying the field', async () => {
    const controls = mount('USD', 'base');
    await click(controls.find((button) => button.title === '点击复制键名 base'));
    expect(copied).toBe('base');
  });

  it('keeps clicking the value unquoted and without its key', async () => {
    const controls = mount('USD', 'base');
    await click(controls.find((button) => button.title === '点击复制 string 值'));
    expect(copied).toBe('USD');
  });

  it('includes an array index of zero', async () => {
    const controls = mount(['USD']);
    await click(controls.find((button) => button.hasAttribute('aria-label')));
    expect(copied).toBe('0: "USD"');
  });

  it('copies a root scalar without inventing a key', async () => {
    const controls = mount('USD');
    await click(controls.find((button) => button.hasAttribute('aria-label')));
    expect(copied).toBe('USD');
  });
});
