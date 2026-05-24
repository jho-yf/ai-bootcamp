<script setup lang="ts">
import type { SlideImage } from '../../types/slide'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  images: SlideImage[]
  activeIndex: number
  slug: string
  isGenerating: boolean
}>()

const emit = defineEmits<{
  select: [index: number]
  delete: [index: number]
  add: []
}>()

const presentationStore = usePresentationStore()

function imageUrl(filename: string) {
  return presentationStore.imageUrl(props.slug, filename)
}
</script>

<template>
  <div class="scrollbar-hidden flex max-h-[calc(100vh-9rem)] flex-col gap-2 overflow-y-auto p-1">
    <div
      v-for="(image, index) in images"
      :key="image.filename"
      class="group relative"
    >
      <button
        class="h-14 w-20 shrink-0 overflow-hidden rounded-lg border-2 bg-white transition"
        :class="index === activeIndex ? 'border-blue-500 shadow-md' : 'border-gray-200 hover:border-gray-300'"
        @click="emit('select', index)"
      >
        <img
          :src="imageUrl(image.filename)"
          class="h-full w-full object-cover"
          :alt="`Thumbnail ${index + 1}`"
        />
      </button>
      <button
        class="absolute right-1 top-1 hidden h-4 w-4 items-center justify-center rounded bg-white/90 text-gray-400 shadow-sm hover:bg-red-50 hover:text-red-500 group-hover:flex"
        @click.stop="emit('delete', index)"
      >
        <svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <button
      class="flex h-14 w-20 shrink-0 items-center justify-center rounded-lg border-2 border-dashed border-gray-300 bg-white/80 text-gray-400 transition hover:border-blue-400 hover:text-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
      :disabled="isGenerating"
      @click="emit('add')"
    >
      <svg v-if="isGenerating" class="h-5 w-5 animate-spin text-blue-500" viewBox="0 0 24 24" fill="none">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
      </svg>
      <svg v-else class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" stroke-linecap="round" />
      </svg>
    </button>
  </div>
</template>
