<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  title?: string
  slug?: string
  editableTitle?: boolean
  cost?: number
}>()

const router = useRouter()
const presentationStore = usePresentationStore()

const isEditing = ref(false)
const editTitle = ref(props.title ?? '')

watch(
  () => props.title,
  (val) => {
    editTitle.value = val ?? ''
  },
)

function startEdit() {
  if (!props.editableTitle) return
  editTitle.value = props.title ?? ''
  isEditing.value = true
}

function saveTitle() {
  if (props.slug && editTitle.value.trim()) {
    presentationStore.updateTitle(props.slug, editTitle.value.trim())
  }
  isEditing.value = false
}

function goHome() {
  router.push('/')
}
</script>

<template>
  <header class="flex h-14 shrink-0 items-center border-b border-gray-200 bg-white px-4">
    <button
      class="flex items-center gap-2 text-lg font-bold text-gray-800 hover:text-blue-600"
      @click="goHome"
    >
      <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-600 text-sm text-white">
        G
      </div>
      GenSlides
    </button>

    <div class="ml-8 flex items-center gap-3">
      <template v-if="editableTitle && isEditing">
        <input
          v-model="editTitle"
          class="w-full max-w-md rounded border border-blue-300 px-2 py-1 text-sm font-medium focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          @keyup.enter="saveTitle"
          @blur="saveTitle"
        />
      </template>
      <template v-else-if="editableTitle">
        <h1
          class="cursor-pointer truncate text-sm font-medium text-gray-700 hover:text-gray-900"
          @click="startEdit"
        >
          {{ title }}
        </h1>
      </template>
      <div
        v-if="cost != null"
        class="flex items-center gap-1.5 rounded-full border border-red-100 bg-red-50 px-2.5 py-1 text-xs font-semibold text-red-600 shadow-sm"
      >
        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span>{{ cost.toFixed(3) }} 元</span>
      </div>
    </div>

    <div class="flex-1" />

    <div class="flex items-center gap-2">
      <slot name="actions" />
    </div>
  </header>
</template>
