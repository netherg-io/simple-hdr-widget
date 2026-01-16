import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function useWindowControls() {
  const isPinned = useLocalStorage('is-pinned', () => false);
  const isHovered = ref(false);
  let unlistenMenu = null;
  let unlistenMouse = null;

  const togglePin = async () => {
    isPinned.value = !isPinned.value;
    await invoke('set_pin_state', { pinned: isPinned.value });
  };

  const onContextMenu = async (e) => {
    e.preventDefault();
    await invoke('show_context_menu', { isPinned: isPinned.value });
  };

  const initWindow = async () => {
    await getCurrentWindow().show();
    await invoke('setup_widget_window');
    await invoke('set_pin_state', { pinned: isPinned.value });
  };

  onMounted(async () => {
    unlistenMenu = await listen('menu-toggle-pin', () => {
      togglePin();
    });

    unlistenMouse = await listen('mouse-left-window', () => {
      isHovered.value = false;
    });

    setTimeout(initWindow, 100);
  });

  onUnmounted(() => {
    if (unlistenMenu) unlistenMenu();
    if (unlistenMouse) unlistenMouse();
  });

  return {
    isPinned,
    isHovered,
    togglePin,
    onContextMenu,
  };
}
