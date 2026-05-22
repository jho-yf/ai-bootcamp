<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  content: string
}>()

const emit = defineEmits<{
  update: [content: string]
}>()

const localContent = ref(props.content)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.content,
  (val) => {
    localContent.value = val
  },
)

function handleInput() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    emit('update', localContent.value)
  }, 500)
}
</script>

<template>
  <textarea
    v-model="localContent"
    class="flex-1 resize-none rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    rows="2"
    placeholder="Enter slide content..."
    @input="handleInput"
  />
</template>
