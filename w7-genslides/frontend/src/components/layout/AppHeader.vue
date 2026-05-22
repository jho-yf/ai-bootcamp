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
      <span v-if="cost != null" class="text-sm font-medium text-red-500">
        ${{ cost.toFixed(3) }}
      </span>
    </div>

    <div class="flex-1" />
  </header>
</template>
