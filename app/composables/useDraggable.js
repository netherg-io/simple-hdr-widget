import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';

export function useDraggable(isPinnedRef) {
  const isDragging = ref(false);
  const hasDragged = ref(false);
  const isMoving = ref(false);

  const dragStartScreenX = ref(0);
  const dragStartScreenY = ref(0);
  const dragOffsetX = ref(0);
  const dragOffsetY = ref(0);
  const minPosX = ref(0);
  const minPosY = ref(0);
  const maxPosX = ref(0);
  const maxPosY = ref(0);

  const restorePosition = async () => {
    try {
      const savedX = localStorage.getItem('hdr_widget_x');
      const savedY = localStorage.getItem('hdr_widget_y');

      if (savedX !== null && savedY !== null) {
        await getCurrentWindow().setPosition(
          new LogicalPosition(parseFloat(savedX), parseFloat(savedY)),
        );
      } else {
        await invoke('init_position');
      }
    } catch (e) {
      console.error('Failed to restore position:', e);
    }
  };

  const onDragMove = async (e) => {
    if (!isDragging.value || isMoving.value) return;

    const dist = Math.sqrt(
      Math.pow(e.screenX - dragStartScreenX.value, 2) +
        Math.pow(e.screenY - dragStartScreenY.value, 2),
    );
    if (dist < 3 && !hasDragged.value) return;

    hasDragged.value = true;
    isMoving.value = true;

    try {
      const scale = window.devicePixelRatio;
      const currentMousePhysicalX = e.screenX * scale;
      const currentMousePhysicalY = e.screenY * scale;

      let rawX = currentMousePhysicalX - dragOffsetX.value;
      let rawY = currentMousePhysicalY - dragOffsetY.value;

      let newX = Math.max(minPosX.value, Math.min(rawX, maxPosX.value));
      let newY = Math.max(minPosY.value, Math.min(rawY, maxPosY.value));

      await invoke('move_widget', {
        x: Math.round(newX),
        y: Math.round(newY),
        isPinned: isPinnedRef.value,
      });

      localStorage.setItem('hdr_widget_x', (newX / scale).toString());
      localStorage.setItem('hdr_widget_y', (newY / scale).toString());
    } catch (err) {
      console.error(err);
    } finally {
      isMoving.value = false;
    }
  };

  const stopDrag = async () => {
    isDragging.value = false;
    document.removeEventListener('mousemove', onDragMove);
    document.removeEventListener('mouseup', stopDrag);

    try {
      const win = getCurrentWindow();
      const factor = await win.scaleFactor();
      const physicalPos = await win.outerPosition();
      const logicalPos = physicalPos.toLogical(factor);

      localStorage.setItem('hdr_widget_x', logicalPos.x.toString());
      localStorage.setItem('hdr_widget_y', logicalPos.y.toString());
    } catch (e) {
      console.error('Failed to save final position', e);
    }
  };

  const startDrag = async (e) => {
    if (e.button !== 0) return;

    const scale = window.devicePixelRatio;
    const windowSize = await getCurrentWindow().outerSize();

    const workAreaLeft = window.screen.availLeft * scale;
    const workAreaTop = window.screen.availTop * scale;
    const workAreaWidth = window.screen.availWidth * scale;
    const workAreaHeight = window.screen.availHeight * scale;

    minPosX.value = Math.round(workAreaLeft);
    minPosY.value = Math.round(workAreaTop);
    maxPosX.value = Math.round(workAreaLeft + workAreaWidth - windowSize.width);
    maxPosY.value = Math.round(
      workAreaTop + workAreaHeight - windowSize.height,
    );

    const physicalWindowPos = await getCurrentWindow().outerPosition();
    const mousePhysicalX = e.screenX * scale;
    const mousePhysicalY = e.screenY * scale;

    dragOffsetX.value = mousePhysicalX - physicalWindowPos.x;
    dragOffsetY.value = mousePhysicalY - physicalWindowPos.y;

    dragStartScreenX.value = e.screenX;
    dragStartScreenY.value = e.screenY;

    hasDragged.value = false;
    isDragging.value = true;

    document.addEventListener('mousemove', onDragMove);
    document.addEventListener('mouseup', stopDrag);
  };

  onMounted(() => {
    setTimeout(restorePosition, 50);
  });

  return {
    startDrag,
    hasDragged,
    isDragging,
  };
}
