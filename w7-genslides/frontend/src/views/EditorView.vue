<script setup lang="ts">
import { onMounted, computed, watch, ref } from 'vue'
import { useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { usePresentationStore } from '../stores/presentation'
import { useUiStore } from '../stores/ui'
import AppHeader from '../components/layout/AppHeader.vue'
import SlideList from '../components/layout/SlideList.vue'
import ImagePreview from '../components/preview/ImagePreview.vue'
import ThumbnailBar from '../components/preview/ThumbnailBar.vue'
import FullscreenPlayer from '../components/preview/FullscreenPlayer.vue'
import SlideEditPopup from '../components/editor/SlideEditPopup.vue'
import ConfirmDialog from '../components/common/ConfirmDialog.vue'
import StyleGuidePopup from '../components/style/StyleGuidePopup.vue'
import StyleThumbnail from '../components/style/StyleThumbnail.vue'
import StyleDetailPopup from '../components/style/StyleDetailPopup.vue'

const route = useRoute()
const presentationStore = usePresentationStore()
const uiStore = useUiStore()
const { presentation, selectedSid, selectedSlide, currentImage, generatingSids, isLoading } =
  storeToRefs(presentationStore)

const slug = computed(() => route.params.slug as string)
const showStyleDetail = ref(false)
const showSlideEdit = ref(false)
const editingSid = ref<string | null>(null)
const showDeleteConfirm = ref(false)
const deletingSid = ref<string | null>(null)

const needsStyleGuide = computed(() => {
  if (!presentation.value) return false
  return presentation.value.style.referenceImage === null
})

const isGenerating = computed(() => {
  if (!selectedSid.value) return false
  return generatingSids.value.has(selectedSid.value)
})

const editingSlide = computed(() => {
  if (!editingSid.value || !presentation.value) return null
  return presentation.value.slides.find((s) => s.sid === editingSid.value) ?? null
})

onMounted(() => {
  presentationStore.loadPresentation(slug.value)
})

watch(needsStyleGuide, (val) => {
  if (val) {
    uiStore.openStyleGuide()
  }
}, { immediate: true })

function handleAddSlide() {
  if (!presentation.value) return
  if (presentation.value.style.referenceImage === null) {
    uiStore.setError('请先生成并选择风格图片')
    return
  }
  presentationStore.addSlide(slug.value, '')
}

function handleContentChange(content: string) {
  if (!editingSid.value) return
  presentationStore.updateSlide(slug.value, editingSid.value, { content })
}

function handleSaveAndGenerate(content: string) {
  if (!editingSid.value) return
  presentationStore.updateSlide(slug.value, editingSid.value, { content })
  presentationStore.generateImage(slug.value, editingSid.value)
}

function handleSlideEdit(sid: string) {
  editingSid.value = sid
  showSlideEdit.value = true
}

function handleDeleteSlide(sid: string) {
  deletingSid.value = sid
  showDeleteConfirm.value = true
}

function confirmDeleteSlide() {
  if (deletingSid.value) {
    presentationStore.deleteSlide(slug.value, deletingSid.value)
  }
  showDeleteConfirm.value = false
  deletingSid.value = null
}

function handleImageSelect(index: number) {
  if (!selectedSid.value) return
  presentationStore.updateSlide(slug.value, selectedSid.value, { activeImageIndex: index })
}

function handleFullscreenStart() {
  const el = document.documentElement
  if (el.requestFullscreen) {
    el.requestFullscreen()
  }
}

function handleStyleGuideClose() {
  uiStore.closeStyleGuide()
}

function handleStyleThumbnailClick() {
  if (!presentation.value) return
  if (presentation.value.style.referenceImage === null) {
    uiStore.openStyleGuide()
  } else {
    showStyleDetail.value = true
  }
}
</script>

<template>
  <div class="flex h-screen flex-col bg-white">
    <AppHeader :title="presentation?.title ?? '加载中...'" :slug="slug" :cost="presentation?.totalCost" editable-title>
      <template #actions>
        <button
          class="flex h-8 w-8 items-center justify-center rounded-lg text-gray-500 transition hover:bg-gray-100 hover:text-gray-700"
          @click="handleFullscreenStart"
        >
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
          </svg>
        </button>
      </template>
    </AppHeader>

    <div v-if="isLoading" class="flex flex-1 items-center justify-center text-gray-400">
      加载演示文稿中...
    </div>

    <div v-else-if="!presentation" class="flex flex-1 items-center justify-center text-gray-400">
      演示文稿不存在
    </div>

    <template v-else>
      <div class="flex flex-1 overflow-hidden">
        <!-- Left: Style Thumbnail + Slide List -->
        <aside class="flex w-60 shrink-0 flex-col border-r border-gray-200 bg-gray-50">
          <div class="px-3 pt-2 pb-1">
            <span class="text-xs font-medium text-gray-500">Style</span>
          </div>
          <StyleThumbnail
            :style-config="presentation.style"
            :slug="slug"
            @click="handleStyleThumbnailClick"
          />
          <SlideList
            :slides="presentation.slides"
            :selected-sid="selectedSid"
            :generating-sids="generatingSids"
            @select="presentationStore.selectSlide"
            @add="handleAddSlide"
            @delete="handleDeleteSlide"
            @edit="handleSlideEdit"
            @reorder="(sids) => presentationStore.reorderSlides(slug, sids)"
          />
        </aside>

        <!-- Right: Preview -->
        <div class="flex flex-1 flex-col overflow-hidden">
          <div class="flex flex-1 flex-col overflow-hidden">
            <ImagePreview
              v-if="selectedSlide"
              :slide="selectedSlide"
              :slug="slug"
              :is-generating="isGenerating"
              @select="handleImageSelect"
              @dblclick="handleSlideEdit(selectedSlide!.sid)"
            />
            <div
              v-else
              class="flex flex-1 items-center justify-center text-gray-400"
            >
              选择一个 Slide 进行预览
            </div>
          </div>

          <!-- Bottom: Thumbnail Bar -->
          <div
            v-if="selectedSlide && selectedSlide.images.length > 0"
            class="flex items-center gap-3 border-t border-gray-100 px-4 py-3"
          >
            <ThumbnailBar
              :images="selectedSlide.images"
              :active-index="selectedSlide.activeImageIndex"
              :slug="slug"
              @select="handleImageSelect"
            />
          </div>
        </div>
      </div>

      <!-- Fullscreen Player -->
      <FullscreenPlayer
        :slides="presentation.slides"
        :slug="slug"
        @start="handleFullscreenStart"
      />

      <!-- Slide Edit Popup -->
      <SlideEditPopup
        :visible="showSlideEdit"
        :content="editingSlide?.content ?? ''"
        :is-generating="isGenerating"
        @close="showSlideEdit = false"
        @update="handleContentChange"
        @save-and-generate="handleSaveAndGenerate"
      />

      <!-- Style Guide Popup -->
      <StyleGuidePopup
        :slug="slug"
        :visible="uiStore.showStyleGuide"
        :style-config="presentation.style"
        @close="handleStyleGuideClose"
      />

      <!-- Delete Confirm Dialog -->
      <ConfirmDialog
        v-model:visible="showDeleteConfirm"
        title="删除 Slide"
        confirm-text="删除"
        variant="danger"
        @confirm="confirmDeleteSlide"
      >
        <p class="text-sm text-gray-500">确定要删除这个 Slide 吗？此操作不可撤销。</p>
      </ConfirmDialog>

      <!-- Style Detail Popup -->
      <StyleDetailPopup
        :visible="showStyleDetail"
        :style-config="presentation.style"
        :slug="slug"
        @close="showStyleDetail = false"
      />
    </template>
  </div>
</template>
