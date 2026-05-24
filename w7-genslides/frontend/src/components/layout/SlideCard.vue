<script setup lang="ts">
import { computed } from 'vue'
import type { Slide } from '../../types/slide'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  slide: Slide
  index: number
  isSelected: boolean
  isGenerating: boolean
}>()

const emit = defineEmits<{
  select: []
  delete: []
  view: []
  create: []
}>()

const presentationStore = usePresentationStore()

const hasImage = computed(() => props.slide.images.length > 0)

const thumbnailUrl = computed(() => {
  if (!hasImage.value) return null
  const image = props.slide.images[props.slide.activeImageIndex]
  if (!image) return null
  const slug = presentationStore.presentation?.slug
  if (!slug) return null
  return presentationStore.imageUrl(slug, image.filename)
})

function handleCreate() {
  if (!hasImage.value && !props.isGenerating) {
    emit('create')
  }
}
</script>

<template>
  <div
    class="group cursor-pointer rounded-lg border bg-white p-2 transition"
    :class="[
      isSelected
        ? 'border-blue-400 ring-2 ring-blue-100'
        : 'border-gray-200 hover:border-gray-300',
    ]"
    :data-selected="isSelected"
    @click="emit('select')"
    @dblclick="handleCreate"
  >
    <div class="mb-1 flex items-center justify-between">
      <span class="text-xs font-medium text-gray-500">幻灯片 {{ index + 1 }}</span>
      <button
        class="hidden h-4 w-4 items-center justify-center rounded text-gray-400 hover:bg-red-50 hover:text-red-500 group-hover:flex"
        @click.stop="emit('delete')"
      >
        <svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="flex h-20 items-center justify-center overflow-hidden rounded bg-gray-50">
      <img
        v-if="thumbnailUrl"
        :src="thumbnailUrl"
        class="h-full w-full object-cover"
      />
      <div v-else-if="isGenerating" class="flex items-center gap-1 text-xs text-blue-500">
        <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
        </svg>
        生成中
      </div>
      <span v-else class="text-xs text-gray-300">暂无图片，双击创建</span>
    </div>

    <button
      class="mt-1 block w-full rounded px-1 py-0.5 text-left text-xs text-gray-600 transition hover:bg-blue-50 hover:text-blue-600"
      @click.stop="emit('view')"
    >
      <span class="line-clamp-2">{{ slide.content || '暂无内容' }}</span>
    </button>
  </div>
</template>
