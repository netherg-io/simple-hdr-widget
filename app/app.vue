<script setup>
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';

const isHdrOn = ref(false);
const isLoading = ref(false);
const errorMsg = ref('');

const isDragging = ref(false);
const hasDragged = ref(false);
const dragStartScreenX = ref(0);
const dragStartScreenY = ref(0);
const dragOffsetX = ref(0);
const dragOffsetY = ref(0);
const isMoving = ref(false);

const startDrag = async (e) => {
  if (e.button !== 0) return;

  const win = getCurrentWindow();
  const factor = await win.scaleFactor();
  const physicalPos = await win.outerPosition();
  const logicalPos = physicalPos.toLogical(factor);

  dragOffsetX.value = e.screenX - logicalPos.x;
  dragOffsetY.value = e.screenY - logicalPos.y;
  dragStartScreenX.value = e.screenX;
  dragStartScreenY.value = e.screenY;
  hasDragged.value = false;
  isDragging.value = true;

  document.addEventListener('mousemove', onDragMove);
  document.addEventListener('mouseup', stopDrag);
};

const onDragMove = async (e) => {
  if (!isDragging.value || isMoving.value) return;

  const dist = Math.hypot(
    e.screenX - dragStartScreenX.value,
    e.screenY - dragStartScreenY.value,
  );
  if (dist > 3) {
    hasDragged.value = true;
  }

  isMoving.value = true;

  try {
    const screenW = window.screen.availWidth;
    const screenH = window.screen.availHeight;
    const widgetW = 140;
    const widgetH = 60;

    let newX = e.screenX - dragOffsetX.value;
    let newY = e.screenY - dragOffsetY.value;
    newX = Math.max(0, Math.min(newX, screenW - widgetW));
    newY = Math.max(0, Math.min(newY, screenH - widgetH));

    await getCurrentWindow().setPosition(new LogicalPosition(newX, newY));
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

const checkStatus = async () => {
  try {
    isHdrOn.value = await invoke('check_hdr_status');
  } catch (e) {
    console.error(e);
  }
};

const toggleHdr = async () => {
  if (hasDragged.value) return;

  if (isLoading.value) return;
  isLoading.value = true;
  try {
    await invoke('toggle_hdr');
    setTimeout(async () => {
      isHdrOn.value = !isHdrOn.value;
      isLoading.value = false;
    }, 2500);
  } catch (e) {
    errorMsg.value = 'Failed to toggle';
    isLoading.value = false;
  }
};

onMounted(async () => {
  await checkStatus();

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
      await getCurrentWindow().setFocus();
    } catch (e) {
      console.error(e);
      await getCurrentWindow().show();
    }
  }, 100);
});
</script>

<template>
  <div class="hdr-widget" @mousedown="startDrag">
    <button
      class="hdr-widget__button"
      :class="{
        'hdr-widget__button--active': isHdrOn,
        'hdr-widget__button--loading': isLoading,
      }"
      :disabled="isLoading"
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
  align-items: center;
  justify-content: center;
  height: 100vh;
  -webkit-app-region: no-drag;

  &__button {
    position: relative;
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px 20px;
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

    &:hover {
      background-color: rgb(50 50 50 / 95%);
      transform: scale(1.02);
    }

    &:active {
      transform: scale(0.98);
    }

    &--active {
      background-color: rgb(0 120 212 / 90%);
      border-color: rgb(255 255 255 / 30%);
      box-shadow: 0 0 15px rgb(0 120 212 / 50%);

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
