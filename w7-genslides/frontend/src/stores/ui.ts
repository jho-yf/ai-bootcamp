import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const error = ref<string | null>(null)
  const success = ref<string | null>(null)
  const isLoading = ref(false)
  const showStyleGuide = ref(false)

  function setError(message: string) {
    error.value = message
  }

  function clearError() {
    error.value = null
  }

  function setSuccess(message: string) {
    success.value = message
  }

  function clearSuccess() {
    success.value = null
  }

  function setLoading(loading: boolean) {
    isLoading.value = loading
  }

  function openStyleGuide() {
    showStyleGuide.value = true
  }

  function closeStyleGuide() {
    showStyleGuide.value = false
  }

  return {
    error,
    success,
    isLoading,
    showStyleGuide,
    setError,
    clearError,
    setSuccess,
    clearSuccess,
    setLoading,
    openStyleGuide,
    closeStyleGuide,
  }
})
