<script setup lang="ts">
import { watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useUiStore } from './stores/ui'
import { usePresentationStore } from './stores/presentation'

const uiStore = useUiStore()
const presentationStore = usePresentationStore()
const { error, success } = storeToRefs(uiStore)
const { error: presentationError } = storeToRefs(presentationStore)

watch(error, (val) => {
  if (val) {
    setTimeout(() => uiStore.clearError(), 3000)
  }
})

watch(success, (val) => {
  if (val) {
    setTimeout(() => uiStore.clearSuccess(), 3000)
  }
})

watch(presentationError, (val) => {
  if (val) {
    uiStore.setError(val)
    presentationError.value = null
    setTimeout(() => uiStore.clearError(), 3000)
  }
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <router-view />

    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="translate-y-2 opacity-0"
        enter-to-class="translate-y-0 opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="translate-y-0 opacity-100"
        leave-to-class="translate-y-2 opacity-0"
      >
        <div
          v-if="error"
          class="fixed top-4 left-1/2 z-50 -translate-x-1/2 rounded-lg bg-red-600 px-4 py-3 text-sm text-white shadow-lg"
        >
          {{ error }}
        </div>
      </Transition>

      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="translate-y-2 opacity-0"
        enter-to-class="translate-y-0 opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="translate-y-0 opacity-100"
        leave-to-class="translate-y-2 opacity-0"
      >
        <div
          v-if="success"
          class="fixed bottom-4 right-4 z-50 rounded-lg bg-green-600 px-4 py-3 text-sm text-white shadow-lg"
        >
          {{ success }}
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
