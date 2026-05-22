<script setup lang="ts">
import { computed } from 'vue'
import type { StyleConfig } from '../../types/style'

const props = defineProps<{
  styleConfig: StyleConfig
  slug: string
}>()

const emit = defineEmits<{
  click: []
}>()

const imageUrl = computed(() => {
  if (!props.styleConfig.referenceImage) return null
  return `/api/ppt/${props.slug}/images/${props.styleConfig.referenceImage}`
})
</script>

<template>
  <div
    class="mx-3 mb-2 cursor-pointer overflow-hidden rounded-lg border border-gray-200 bg-white transition hover:border-blue-400 hover:shadow-sm"
    @click="emit('click')"
  >
    <div v-if="imageUrl" class="relative aspect-video w-full bg-gray-100">
      <img :src="imageUrl" alt="风格图片" class="h-full w-full object-cover" />
    </div>
    <div v-else class="flex aspect-video w-full items-center justify-center bg-gray-100 text-gray-400">
      <svg class="mr-1 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z" />
      </svg>
      <span class="text-xs">No picture</span>
    </div>
  </div>
</template>
