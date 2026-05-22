<script setup lang="ts">
import type { SlideImage } from '../../types/slide'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  images: SlideImage[]
  activeIndex: number
  slug: string
}>()

const emit = defineEmits<{
  select: [index: number]
}>()

const presentationStore = usePresentationStore()

function imageUrl(filename: string) {
  return presentationStore.imageUrl(props.slug, filename)
}
</script>

<template>
  <div class="flex gap-2 overflow-x-auto">
    <button
      v-for="(image, index) in images"
      :key="image.filename"
      class="h-14 w-20 shrink-0 overflow-hidden rounded border-2 transition"
      :class="index === activeIndex ? 'border-blue-500' : 'border-gray-200 hover:border-gray-300'"
      @click="emit('select', index)"
    >
      <img
        :src="imageUrl(image.filename)"
        class="h-full w-full object-cover"
        :alt="`Thumbnail ${index + 1}`"
      />
    </button>
  </div>
</template>
