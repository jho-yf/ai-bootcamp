<script setup lang="ts">
import type { StyleConfig } from '../../types/style'

defineProps<{
  visible: boolean
  styleConfig: StyleConfig
  slug: string
}>()

const emit = defineEmits<{
  close: []
}>()

function imageUrl(slug: string, filename: string) {
  return `/api/ppt/${slug}/images/${filename}`
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="visible" class="fixed inset-0 z-40 flex items-center justify-center bg-black/50" @click="emit('close')">
        <div class="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl" @click.stop>
          <h2 class="mb-4 text-lg font-bold text-gray-800">当前风格</h2>
          <div v-if="styleConfig.referenceImage" class="mb-4 overflow-hidden rounded-lg">
            <img :src="imageUrl(slug, styleConfig.referenceImage)" alt="风格图片" class="w-full object-cover" />
          </div>
          <div v-if="styleConfig.prompt" class="mb-4">
            <p class="mb-1 text-xs font-medium text-gray-500">提示词</p>
            <p class="rounded-lg bg-gray-50 p-3 text-sm text-gray-700">{{ styleConfig.prompt }}</p>
          </div>
          <div class="flex justify-end">
            <button
              class="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
              @click="emit('close')"
            >
              关闭
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
