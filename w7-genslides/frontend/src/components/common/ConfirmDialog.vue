<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  visible: boolean
  title: string
  confirmText?: string
  variant?: 'primary' | 'danger'
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: []
}>()

function close() {
  emit('update:visible', false)
}

const confirmClass = computed(() => {
  const base = 'rounded-lg px-4 py-2 text-sm font-medium text-white '
  return props.variant === 'danger'
    ? base + 'bg-red-600 hover:bg-red-700'
    : base + 'bg-blue-600 hover:bg-blue-700'
})
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
          class="w-full max-w-sm rounded-2xl bg-white p-6 shadow-2xl"
          @click.stop
        >
          <h3 class="mb-4 text-lg font-bold text-gray-800">{{ title }}</h3>
          <div class="mb-6">
            <slot />
          </div>
          <div class="flex justify-end gap-2">
            <button
              class="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
              @click="close"
            >
              取消
            </button>
            <button
              :class="confirmClass"
              @click="emit('confirm')"
            >
              {{ confirmText ?? '确定' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
