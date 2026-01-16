<script setup>
defineProps({
  src: {
    type: String,
    required: true,
  },
  srcset: {
    type: String,
    default: undefined,
  },
  alt: {
    type: String,
    default: '',
  },
  loading: {
    type: String,
    default: 'lazy',
    validator: (value) => ['lazy', 'eager'].includes(value),
  },
  fetchPriority: {
    type: String,
    default: undefined,
    validator: (value) => ['high', 'low', 'auto'].includes(value),
  },
});
</script>

<template>
  <img
    :src="src"
    :alt="alt"
    :loading="loading"
    :fetchpriority="fetchPriority"
  />
</template>
