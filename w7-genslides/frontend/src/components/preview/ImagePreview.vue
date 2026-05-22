<script setup lang="ts">
import { computed } from 'vue'
import type { Slide } from '../../types/slide'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  slide: Slide
  slug: string
  isGenerating: boolean
}>()

const emit = defineEmits<{
  select: [index: number]
  dblclick: []
}>()

const presentationStore = usePresentationStore()

const imageUrl = computed(() => {
  if (props.slide.images.length === 0) return null
  const image = props.slide.images[props.slide.activeImageIndex]
  if (!image) return null
  return presentationStore.imageUrl(props.slug, image.filename)
})
</script>

<template>
  <div class="flex flex-1 items-center justify-center bg-gray-100 p-4">
    <div class="relative flex h-full w-full items-center justify-center">
      <img
        v-if="imageUrl"
        :src="imageUrl"
        class="max-h-full max-w-full rounded-lg object-contain shadow-sm"
      />

      <div
        v-else-if="isGenerating"
        class="flex flex-col items-center gap-3 text-gray-400"
      >
        <svg class="h-12 w-12 animate-spin text-blue-500" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
        </svg>
        <span class="text-sm">生成图片中...</span>
      </div>

      <div
        v-else
        class="flex cursor-pointer flex-col items-center gap-2 text-gray-300 transition hover:text-gray-400"
        @dblclick="emit('dblclick')"
      >
        <svg class="h-16 w-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1"
            d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
          />
        </svg>
        <span class="text-sm">暂无图片，双击创建</span>
      </div>
    </div>
  </div>
</template>
