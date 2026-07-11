import {
  useEffect,
  useRef,
  useState,
  createContext,
  useContext,
  createElement,
  type ReactNode,
} from "react";

export type AppViewKind = "desktop" | "compact" | "phone";

export type WindowSize = { width: number; height: number };

export const APP_VIEW_WINDOW_SIZES: Record<AppViewKind, WindowSize & { label: string }> = {
  desktop: { width: 1100, height: 720, label: "1100×720" },
  compact: { width: 900, height: 700, label: "900×700" },
  phone: { width: 430, height: 780, label: "430×780" },
};

export function normalizeAppView(value: unknown): AppViewKind {
  if (value === "compact" || value === "phone" || value === "desktop") {
    return value;
  }
  // Legacy "auto" (and anything else) → current window, or desktop.
  if (typeof window !== "undefined") {
    return layoutFromWidth(window.innerWidth);
  }
  return "desktop";
}

export function layoutFromWidth(width: number): AppViewKind {
  if (width < 768) return "phone";
  if (width < 1024) return "compact";
  return "desktop";
}

function clampSize(width: number, height: number): WindowSize {
  return {
    width: Math.max(420, Math.round(width)),
    height: Math.max(560, Math.round(height)),
  };
}

/** Resolve size to restore: saved pixels, else the view preset. */
export function resolveWindowSize(settings: {
  app_view?: unknown;
  window_width?: number | null;
  window_height?: number | null;
}): WindowSize {
  const view = normalizeAppView(settings.app_view);
  const preset = APP_VIEW_WINDOW_SIZES[view];
  const width = settings.window_width ?? preset.width;
  const height = settings.window_height ?? preset.height;
  return clampSize(width, height);
}

/** While true, ignore resize→settings sync (programmatic size change in progress). */
let suppressViewSyncUntil = 0;

export function suppressAppViewResizeSync(ms = 400) {
  suppressViewSyncUntil = Date.now() + ms;
}

export function isAppViewResizeSyncSuppressed() {
  return Date.now() < suppressViewSyncUntil;
}

export async function applyWindowSize(width: number, height: number): Promise<void> {
  const size = clampSize(width, height);
  suppressAppViewResizeSync(500);
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const { LogicalSize } = await import("@tauri-apps/api/dpi");
    const win = getCurrentWindow();
    if (await win.isMaximized()) {
      await win.unmaximize();
    }
    await win.setSize(new LogicalSize(size.width, size.height));
  } catch {
    // No-op in browser preview or if window APIs are unavailable.
  }
}

/** Resize the native window to the default size for a view preference. */
export async function applyAppViewWindowSize(view: AppViewKind): Promise<void> {
  const size = APP_VIEW_WINDOW_SIZES[view];
  await applyWindowSize(size.width, size.height);
}

/** Restore last saved window size (or the view preset if none saved). */
export async function restoreSavedWindowSize(settings: {
  app_view?: unknown;
  window_width?: number | null;
  window_height?: number | null;
}): Promise<void> {
  const size = resolveWindowSize(settings);
  await applyWindowSize(size.width, size.height);
}

const LayoutModeContext = createContext<AppViewKind>("desktop");

function useLayoutModeState(): AppViewKind {
  const [layout, setLayout] = useState<AppViewKind>(() =>
    typeof window === "undefined" ? "desktop" : layoutFromWidth(window.innerWidth),
  );

  useEffect(() => {
    const sync = () => setLayout(layoutFromWidth(window.innerWidth));
    sync();
    window.addEventListener("resize", sync);
    return () => window.removeEventListener("resize", sync);
  }, []);

  return layout;
}

/** Single shared resize→layout listener for the app shell. */
export function LayoutModeProvider({ children }: { children: ReactNode }) {
  const layout = useLayoutModeState();
  return createElement(LayoutModeContext.Provider, { value: layout }, children);
}

/** Chrome layout always follows current window width. */
export function useLayoutMode(): AppViewKind {
  return useContext(LayoutModeContext);
}

export type WindowLayoutState = {
  view: AppViewKind;
  width: number;
  height: number;
};

/** Persist app_view + exact window size whenever the user resizes. */
export function useSyncAppViewOnResize(
  appView: AppViewKind,
  savedWidth: number | null | undefined,
  savedHeight: number | null | undefined,
  onChange: (next: WindowLayoutState) => void | Promise<void>,
) {
  const appViewRef = useRef(appView);
  const sizeRef = useRef({ width: savedWidth, height: savedHeight });
  const onChangeRef = useRef(onChange);
  appViewRef.current = appView;
  sizeRef.current = { width: savedWidth, height: savedHeight };
  onChangeRef.current = onChange;

  useEffect(() => {
    let timer: number | undefined;
    const onResize = () => {
      if (isAppViewResizeSyncSuppressed()) return;
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        if (isAppViewResizeSyncSuppressed()) return;
        const width = Math.round(window.innerWidth);
        const height = Math.round(window.innerHeight);
        const view = layoutFromWidth(width);
        const prev = sizeRef.current;
        if (
          view === appViewRef.current &&
          prev.width === width &&
          prev.height === height
        ) {
          return;
        }
        void onChangeRef.current({ view, width, height });
      }, 300);
    };
    window.addEventListener("resize", onResize);
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("resize", onResize);
    };
  }, []);
}
