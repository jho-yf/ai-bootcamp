<script setup lang="ts">
import { ref, computed, watch } from 'vue'

const props = defineProps<{
  visible: boolean
  content: string
  isGenerating: boolean
}>()

const emit = defineEmits<{
  close: []
  update: [content: string]
  saveAndGenerate: [content: string]
}>()

const localContent = ref('')

const canSave = computed(() => localContent.value.trim().length > 0)

watch(() => props.visible, (val) => {
  if (val) localContent.value = props.content
})

function handleSave() {
  emit('update', localContent.value)
  emit('close')
}

function handleSaveAndGenerate() {
  emit('saveAndGenerate', localContent.value)
  emit('close')
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
        <div
          class="w-full max-w-2xl rounded-2xl bg-white p-6 shadow-2xl"
          @click.stop
        >
          <h2 class="mb-4 text-lg font-bold text-gray-800">编辑 Slide 内容</h2>
          <textarea
            v-model="localContent"
            class="mb-4 w-full resize-none rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-gray-50 disabled:text-gray-400"
            rows="10"
            placeholder="输入 slide 内容..."
            :disabled="isGenerating"
          />
          <div class="flex justify-end gap-2">
            <button
              class="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
              @click="emit('close')"
            >
              取消
            </button>
            <button
              class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              :disabled="isGenerating || !canSave"
              @click="handleSave"
            >
              仅保存
            </button>
            <button
              class="rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:opacity-50"
              :disabled="isGenerating || !canSave"
              @click="handleSaveAndGenerate"
            >
              {{ isGenerating ? '生成中...' : '保存并生成' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
