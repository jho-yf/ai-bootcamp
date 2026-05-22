<script setup lang="ts">
import { ref } from 'vue'
import type { StyleConfig } from '../../types/style'
import { usePresentationStore } from '../../stores/presentation'
import StyleSelector from './StyleSelector.vue'

const props = defineProps<{
  slug: string
  visible: boolean
  styleConfig: StyleConfig
}>()

const emit = defineEmits<{
  close: []
}>()

const presentationStore = usePresentationStore()

const step = ref<'prompt' | 'select'>('prompt')
const prompt = ref('')
const candidates = ref<string[]>([])
const isGenerating = ref(false)

async function handleGenerate() {
  if (!prompt.value.trim()) return
  isGenerating.value = true
  const result = await presentationStore.generateStyleCandidates(props.slug, prompt.value.trim())
  isGenerating.value = false
  if (result) {
    candidates.value = result.candidates
    if (result.candidates.length > 0) {
      step.value = 'select'
    }
  }
}

async function handleSelect(referenceImage: string) {
  await presentationStore.selectStyle(props.slug, referenceImage)
  emit('close')
  step.value = 'prompt'
  prompt.value = ''
  candidates.value = []
}

function handleClose() {
  if (candidates.value.length > 0) {
    presentationStore.cleanupCandidates(props.slug)
  }
  emit('close')
  step.value = 'prompt'
  prompt.value = ''
  candidates.value = []
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
      <div v-if="visible" class="fixed inset-0 z-40 flex items-center justify-center bg-black/50">
        <div
          class="w-full max-w-xl rounded-2xl bg-white p-6 shadow-2xl"
          @click.stop
        >
          <h2 class="mb-4 text-lg font-bold text-gray-800">风格引导</h2>
          <p class="mb-4 text-sm text-gray-500">
            描述你想要的演示文稿图片视觉风格。
          </p>

          <template v-if="step === 'prompt'">
            <textarea
              v-model="prompt"
              class="mb-4 w-full resize-none rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:bg-gray-50 disabled:text-gray-400"
              rows="3"
              placeholder="例如：现代极简风格，柔和的粉彩色调，简洁的线条..."
              :disabled="isGenerating"
            />
            <div class="flex justify-end gap-2">
              <button
                class="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
                @click="handleClose"
              >
                取消
              </button>
              <button
                class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
                :disabled="!prompt.trim() || isGenerating"
                @click="handleGenerate"
              >
                {{ isGenerating ? '生成中...' : '生成候选' }}
              </button>
            </div>
          </template>

          <template v-else>
            <StyleSelector
              :candidates="candidates"
              :slug="slug"
              @select="handleSelect"
            />
            <div class="mt-4 flex justify-end">
              <button
                class="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
                @click="presentationStore.cleanupCandidates(slug); step = 'prompt'"
              >
                返回
              </button>
            </div>
          </template>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
