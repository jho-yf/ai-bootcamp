<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { usePresentationStore } from '../stores/presentation'
import AppHeader from '../components/layout/AppHeader.vue'
import ConfirmDialog from '../components/common/ConfirmDialog.vue'

const router = useRouter()
const presentationStore = usePresentationStore()
const { presentations, isLoading } = storeToRefs(presentationStore)

const showCreateDialog = ref(false)
const showDeleteDialog = ref(false)
const targetSlug = ref('')
const targetTitle = ref('')
const deleteTargetSlug = ref('')

const createTitle = ref('')

onMounted(() => {
  presentationStore.loadPresentations()
})

function openEditor(slug: string) {
  router.push(`/${slug}`)
}

function handleCreate() {
  if (!createTitle.value.trim()) return
  presentationStore.createPresentation(createTitle.value.trim())
  showCreateDialog.value = false
  createTitle.value = ''
}

function confirmDelete(slug: string, title: string) {
  deleteTargetSlug.value = slug
  targetTitle.value = title
  showDeleteDialog.value = true
}

function handleDelete() {
  presentationStore.deletePresentation(deleteTargetSlug.value)
  showDeleteDialog.value = false
  deleteTargetSlug.value = ''
}
</script>

<template>
  <div class="flex min-h-screen flex-col">
    <AppHeader />

    <main class="mx-auto w-full max-w-4xl flex-1 px-6 py-8">
      <div class="mb-6 flex items-center justify-between">
        <h1 class="text-2xl font-bold text-gray-800">我的演示文稿</h1>
        <button
          class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 active:bg-blue-800"
          @click="showCreateDialog = true"
        >
          + 新建演示文稿
        </button>
      </div>

      <div v-if="isLoading" class="py-12 text-center text-gray-400">加载中...</div>

      <div v-else-if="presentations.length === 0" class="py-12 text-center text-gray-400">
        暂无演示文稿，点击上方按钮创建
      </div>

      <div v-else class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <div
          v-for="p in presentations"
          :key="p.slug"
          class="group cursor-pointer rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition hover:shadow-md"
          @click="openEditor(p.slug)"
        >
          <div class="mb-2 flex items-start justify-between">
            <h2 class="truncate text-lg font-semibold text-gray-800">{{ p.title }}</h2>
            <button
              class="ml-2 shrink-0 rounded p-1 text-gray-300 opacity-0 transition hover:bg-red-50 hover:text-red-500 group-hover:opacity-100"
              @click.stop="confirmDelete(p.slug, p.title)"
            >
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                />
              </svg>
            </button>
          </div>
          <div class="flex items-center gap-4 text-sm text-gray-500">
            <span>{{ p.slideCount }} 张幻灯片</span>
            <span>{{ p.totalCost.toFixed(2) }} 元</span>
          </div>
          <div class="mt-2 text-xs text-gray-400">
            {{ new Date(p.updatedAt).toLocaleDateString() }}
          </div>
        </div>
      </div>
    </main>

    <ConfirmDialog
      v-model:visible="showCreateDialog"
      title="新建演示文稿"
      confirm-text="创建"
      @confirm="handleCreate"
    >
      <div class="space-y-3">
        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">标题</label>
          <input
            v-model="createTitle"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            placeholder="我的演示文稿"
          />
        </div>
      </div>
    </ConfirmDialog>

    <ConfirmDialog
      v-model:visible="showDeleteDialog"
      title="删除演示文稿"
      confirm-text="删除"
      variant="danger"
      @confirm="handleDelete"
    >
      <p class="text-sm text-gray-600">
        确定要删除"{{ targetTitle }}"吗？此操作不可撤销。
      </p>
    </ConfirmDialog>
  </div>
</template>
