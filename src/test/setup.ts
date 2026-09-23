import "@testing-library/jest-dom/vitest";

// Stub Tauri globals so @tauri-apps/api imports don't crash in jsdom.
const g = globalThis as Record<string, unknown>;
if (!g["__TAURI__"]) {
  g["__TAURI__"] = {};
}

// jsdom lacks URL.createObjectURL; provide a stub for saveFile tests.
if (typeof URL !== "undefined" && !URL.createObjectURL) {
  URL.createObjectURL = (() => "blob:mock") as typeof URL.createObjectURL;
  URL.revokeObjectURL = (() => undefined) as typeof URL.revokeObjectURL;
}

// matchMedia is used by theme init; stub it.
if (typeof window !== "undefined" && !window.matchMedia) {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => undefined,
    removeListener: () => undefined,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
}
