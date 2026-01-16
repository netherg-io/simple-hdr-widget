<script setup>
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';

const isHdrOn = ref(false);
const isLoading = ref(false);
const isHovered = ref(false);
const isDragging = ref(false);
const hasDragged = ref(false);
const dragStartScreenX = ref(0);
const dragStartScreenY = ref(0);
const dragOffsetX = ref(0);
const dragOffsetY = ref(0);
const minPosX = ref(0);
const minPosY = ref(0);
const maxPosX = ref(0);
const maxPosY = ref(0);
const isMoving = ref(false);

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
  maxPosY.value = Math.round(workAreaTop + workAreaHeight - windowSize.height);

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

    await invoke('move_widget', { x: Math.round(newX), y: Math.round(newY) });

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
    console.error('Failed to save position', e);
  }
};

const toggleHdr = async () => {
  if (hasDragged.value || isLoading.value) return;

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

let unlisten = null;

onMounted(async () => {
  try {
    isHdrOn.value = await invoke('check_hdr_status');
  } catch (e) {
    console.error(e);
  }

  unlisten = await listen('hdr-state-changed', (event) => {
    console.log('HDR State Event:', event.payload.enabled);

    if (!isLoading.value) {
      isHdrOn.value = event.payload.enabled;
    }
  });

  await listen('mouse-left-window', () => {
    isHovered.value = false;
  });

  setTimeout(async () => {
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

      await getCurrentWindow().show();

      await invoke('apply_widget_styles');
    } catch (e) {
      console.error(e);
      await getCurrentWindow().show();
    }
  }, 100);
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});
</script>

<template>
  <div class="hdr-widget" @mousedown="startDrag">
    <button
      class="hdr-widget__button"
      :class="{
        'hdr-widget__button--active': isHdrOn,
        'hdr-widget__button--loading': isLoading,
        'hdr-widget__button--hovered': isHovered,
      }"
      :disabled="isLoading"
      @mouseenter="isHovered = true"
      @mouseleave="isHovered = false"
      @click="toggleHdr"
    >
      <div class="hdr-widget__indicator"></div>

      <span class="hdr-widget__label"> HDR {{ isHdrOn ? 'ON' : 'OFF' }}</span>
    </button>
  </div>
</template>

<style lang="scss" scoped>
.hdr-widget {
  display: flex;
  align-items: stretch;
  justify-content: stretch;
  height: 100vh;
  -webkit-app-region: no-drag;

  &__button {
    position: relative;
    display: flex;
    gap: 10px;
    align-items: center;
    width: 100%;
    padding: 10px 20px;
    margin: 10px;
    overflow: hidden;
    color: #ffffff;
    cursor: pointer;
    user-select: none;
    outline: none;
    background-color: rgb(30 30 30 / 90%);
    border: 1px solid rgb(255 255 255 / 10%);
    border-radius: 30px;
    backdrop-filter: blur(10px);
    transition: all 0.3s ease;
    will-change: transform;

    @include active {
      transform: scale(0.98) !important;
    }

    &--hovered {
      background-color: rgb(50 50 50 / 95%);
      transform: scale(1.02);
    }

    &--active {
      background-color: rgb(0 120 212 / 90%);
      border-color: rgb(255 255 255 / 30%);

      .hdr-widget__indicator {
        background-color: #ffffff;
        box-shadow: 0 0 5px #ffffff;
      }
    }

    &--loading {
      pointer-events: none;
      cursor: wait;
      opacity: 0.8;
    }
  }

  &__indicator {
    width: 8px;
    height: 8px;
    background-color: #555555;
    border-radius: 50%;
    transition:
      background-color 0.3s ease,
      box-shadow 0.3s ease;
  }

  &__label {
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.5px;
  }
}
</style>
