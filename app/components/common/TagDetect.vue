<script setup>
const props = defineProps({
  to: {
    type: [Object, String],
    default: undefined,
  },
  href: {
    type: String,
    default: undefined,
  },
  target: {
    type: String,
    default: undefined,
  },
  isStaticByDefault: {
    type: Boolean,
    default: false,
  },
  staticTag: {
    type: String,
    default: 'div',
  },
  type: {
    type: String,
    default: 'button',
  },
});

const component = computed(() => {
  let element;
  const attrs = {};

  switch (true) {
    case !!props.to:
      element = resolveComponent('NuxtLink'); // https://nuxt.com/docs/guide/directory-structure/components#dynamic-components
      attrs.to = props.to;
      attrs.target = props.target;
      break;
    case !!props.href:
      element = 'a';
      attrs.href = props.href;
      attrs.target = props.target;
      break;
    case props.isStaticByDefault:
      element = props.staticTag;
      break;
    default:
      element = 'button';
      attrs.type = props.type;
  }

  return {
    element,
    attrs,
  };
});
</script>

<template>
  <component :is="component.element" v-bind="component.attrs">
    <slot />
  </component>
</template>
