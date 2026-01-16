import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function useHdr() {
  const isHdrOn = ref(false);
  const isLoading = ref(false);
  let unlisten = null;

  const checkStatus = async () => {
    try {
      isHdrOn.value = await invoke('check_hdr_status');
    } catch (e) {
      console.error('Failed to check HDR status:', e);
    }
  };

  const toggleHdr = async () => {
    if (isLoading.value) return;

    isLoading.value = true;
    try {
      await invoke('toggle_hdr', { enable: !isHdrOn.value });
      isHdrOn.value = !isHdrOn.value;
    } catch (e) {
      console.error('Toggle failed:', e);
    } finally {
      isLoading.value = false;
    }
  };

  onMounted(async () => {
    await checkStatus();

    unlisten = await listen('hdr-state-changed', (event) => {
      console.log('HDR State Event:', event.payload.enabled);
      if (!isLoading.value) {
        isHdrOn.value = event.payload.enabled;
      }
    });
  });

  onUnmounted(() => {
    if (unlisten) unlisten();
  });

  return {
    isHdrOn,
    isLoading,
    toggleHdr,
  };
}
