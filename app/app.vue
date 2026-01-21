<script setup>
const { isPinned, isDragEnabled, isHovered, onContextMenu } =
  useWindowControls();

const { isHdrOn, isLoading, toggleHdr } = useHdr();

const { startDrag, hasDragged } = useDraggable(isPinned, isDragEnabled);

const onButtonClick = () => {
  if (!hasDragged.value) {
    toggleHdr();
  }
};
</script>

<template>
  <div class="hdr-widget" @mousedown="startDrag" @contextmenu="onContextMenu">
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
      @click="onButtonClick"
    >
      <div class="hdr-widget__indicator"></div>
      <span class="hdr-widget__label"> HDR {{ isHdrOn ? 'ON' : 'OFF' }}</span>
    </button>
  </div>
</template>

<style lang="scss" scoped>
.hdr-widget {
  $parent: &;

  display: flex;
  align-items: stretch;
  justify-content: stretch;
  height: 100vh;
  -webkit-app-region: no-drag;

  &__button {
    position: relative;
    display: flex;
    gap: em(10);
    align-items: center;
    width: 100%;
    padding: em(10) em(20);
    margin: em(10);
    overflow: hidden;
    color: #ffffff;
    cursor: pointer;
    user-select: none;
    outline: none;
    background-color: rgb(30 30 30 / 90%);
    border: 1px solid rgb(255 255 255 / 10%);
    border-radius: em(30);
    backdrop-filter: blur(em(10));
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

      #{$parent}__indicator {
        background-color: #ffffff;
        box-shadow: 0 0 em(5) #ffffff;
      }
    }

    &--loading {
      pointer-events: none;
      cursor: wait;
      opacity: 0.8;
    }
  }

  &__indicator {
    width: em(8);
    height: em(8);
    background-color: #555555;
    border-radius: 50%;
    transition:
      background-color 0.3s ease,
      box-shadow 0.3s ease;
  }

  &__label {
    font-size: em(14);
    font-weight: 600;
    letter-spacing: 0.5px;
  }
}
</style>
