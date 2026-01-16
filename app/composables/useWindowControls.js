import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function useWindowControls() {
  const isPinned = useLocalStorage('is-pinned', () => false);
  const isDragEnabled = useLocalStorage('is-drag-enabled', () => true);
  const savedMonitorName = useLocalStorage('saved-monitor-name', () => null);

  const isHovered = ref(false);

  let unlistenMenuPin = null;
  let unlistenMenuDrag = null;
  let unlistenMonitorChange = null;
  let unlistenMouse = null;

  const togglePin = async () => {
    isPinned.value = !isPinned.value;
    await invoke('set_pin_state', { pinned: isPinned.value });
  };

  const onContextMenu = async (e) => {
    e.preventDefault();
    await invoke('show_context_menu', {
      isPinned: isPinned.value,
      isDraggable: isDragEnabled.value,
    });
  };

  const initWindow = async () => {
    const savedX = localStorage.getItem('hdr_widget_x');
    const savedY = localStorage.getItem('hdr_widget_y');

    await invoke('restore_window', {
      savedMonitorName: savedMonitorName.value,
      savedX: savedX ? parseFloat(savedX) : null,
      savedY: savedY ? parseFloat(savedY) : null,
    });

    await getCurrentWindow().show();
    await invoke('setup_widget_window');
    await invoke('set_pin_state', { pinned: isPinned.value });
  };

  onMounted(async () => {
    unlistenMenuPin = await listen('menu-toggle-pin', () => togglePin());

    unlistenMenuDrag = await listen('menu-toggle-drag', () => {
      isDragEnabled.value = !isDragEnabled.value;
    });

    unlistenMonitorChange = await listen('monitor-changed', (event) => {
      savedMonitorName.value = event.payload;
      localStorage.removeItem('hdr_widget_x');
      localStorage.removeItem('hdr_widget_y');
    });

    unlistenMouse = await listen('mouse-left-window', () => {
      isHovered.value = false;
    });

    setTimeout(initWindow, 100);
  });

  onUnmounted(() => {
    if (unlistenMenuPin) unlistenMenuPin();
    if (unlistenMenuDrag) unlistenMenuDrag();
    if (unlistenMonitorChange) unlistenMonitorChange();
    if (unlistenMouse) unlistenMouse();
  });

  return {
    isPinned,
    isDragEnabled,
    isHovered,
    onContextMenu,
  };
}
