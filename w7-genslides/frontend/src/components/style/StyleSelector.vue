<script setup lang="ts">
import { ref } from 'vue'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  candidates: string[]
  slug: string
}>()

const emit = defineEmits<{
  select: [referenceImage: string]
}>()

const presentationStore = usePresentationStore()
const selectedIndex = ref<number | null>(null)

function selectCandidate(index: number) {
  selectedIndex.value = index
}

function confirmSelection() {
  if (selectedIndex.value === null) return
  emit('select', props.candidates[selectedIndex.value])
}
</script>

<template>
  <div>
    <p class="mb-3 text-sm text-gray-600">选择一个参考风格：</p>
    <div class="mb-4 flex justify-center">
      <button
        v-for="(candidate, index) in candidates"
        :key="index"
        class="mx-auto w-full max-w-md overflow-hidden rounded-lg border-2 transition"
        :class="selectedIndex === index ? 'border-blue-500 ring-2 ring-blue-100' : 'border-gray-200 hover:border-gray-300'"
        @click="selectCandidate(index)"
      >
        <img
          :src="presentationStore.imageUrl(slug, candidate)"
          class="h-56 w-full object-cover"
          :alt="`风格选项 ${index + 1}`"
        />
      </button>
    </div>
    <button
      class="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
      :disabled="selectedIndex === null"
      @click="confirmSelection"
    >
      使用此风格
    </button>
  </div>
</template>
