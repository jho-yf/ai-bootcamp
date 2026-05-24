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
const showContentView = ref(false)
const viewingSid = ref<string | null>(null)
const showPlayer = ref(false)

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

const viewingSlide = computed(() => {
  if (!viewingSid.value || !presentation.value) return null
  return presentation.value.slides.find((s) => s.sid === viewingSid.value) ?? null
})

onMounted(() => {
  presentationStore.loadPresentation(slug.value)
})

watch(needsStyleGuide, (val) => {
  if (val) {
    uiStore.openStyleGuide()
  }
}, { immediate: true })

async function handleAddSlide() {
  if (!presentation.value) return
  if (presentation.value.style.referenceImage === null) {
    uiStore.setError('请先生成并选择风格图片')
    return
  }
  const slide = await presentationStore.addSlide(slug.value, '')
  if (slide) {
    presentationStore.selectSlide(slide.sid)
    handleSlideEdit(slide.sid)
  }
}

function handleContentChange(content: string) {
  if (!editingSid.value) return
  presentationStore.updateSlide(slug.value, editingSid.value, { content })
}

async function handleSaveAndGenerate(content: string) {
  if (!editingSid.value) return
  const sid = editingSid.value
  const result = await presentationStore.updateSlide(slug.value, sid, { content })
  if (result) {
    presentationStore.generateImage(slug.value, sid)
  }
}

function handleSlideEdit(sid: string) {
  editingSid.value = sid
  showSlideEdit.value = true
}

function handleSlideView(sid: string) {
  viewingSid.value = sid
  showContentView.value = true
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

function handleImageDelete(index: number) {
  if (!selectedSid.value) return
  presentationStore.deleteSlideImage(slug.value, selectedSid.value, index)
}

function handleFullscreenStart() {
  showPlayer.value = true
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
          class="flex h-8 items-center gap-1.5 rounded-lg bg-blue-600 px-3 text-sm font-medium text-white shadow-sm transition hover:bg-blue-700"
          @click="handleFullscreenStart"
        >
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-5.197-3.03A1 1 0 008 9.002v5.996a1 1 0 001.555.832l5.197-2.966a1 1 0 000-1.696z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          播放
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
        <aside class="flex w-52 shrink-0 flex-col border-r border-gray-200 bg-gray-50">
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
            @view="handleSlideView"
            @create="handleSlideEdit"
            @reorder="(sids) => presentationStore.reorderSlides(slug, sids)"
          />
        </aside>

        <!-- Right: Preview -->
        <div class="relative flex flex-1 flex-col overflow-hidden">
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

          <div
            v-if="selectedSlide && selectedSlide.images.length > 0"
            class="absolute right-4 top-1/2 -translate-y-1/2 rounded-2xl border border-gray-200 bg-white/90 p-2 shadow-xl backdrop-blur"
          >
            <ThumbnailBar
              :images="selectedSlide.images"
              :active-index="selectedSlide.activeImageIndex"
              :slug="slug"
              :is-generating="isGenerating"
              @select="handleImageSelect"
              @delete="handleImageDelete"
              @add="selectedSid && handleSlideEdit(selectedSid)"
            />
          </div>
        </div>
      </div>

      <!-- Fullscreen Player -->
      <FullscreenPlayer
        :slides="presentation.slides"
        :slug="slug"
        :visible="showPlayer"
        @close="showPlayer = false"
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

      <Teleport to="body">
        <Transition
          enter-active-class="transition duration-200 ease-out"
          enter-from-class="opacity-0"
          enter-to-class="opacity-100"
          leave-active-class="transition duration-150 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <div v-if="showContentView" class="fixed inset-0 z-40 flex items-center justify-center bg-black/50" @click="showContentView = false">
            <div class="w-full max-w-4xl rounded-2xl bg-white p-6 shadow-2xl" @click.stop>
              <div class="mb-4 flex items-center justify-between">
                <h2 class="text-lg font-bold text-gray-800">Slide 内容</h2>
                <button
                  class="flex h-7 w-7 items-center justify-center rounded text-gray-400 hover:bg-gray-100 hover:text-gray-600"
                  @click="showContentView = false"
                >
                  <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 6L6 18M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <div class="max-h-[70vh] overflow-y-auto whitespace-pre-wrap rounded-xl bg-gray-50 p-5 text-sm leading-6 text-gray-700">
                {{ viewingSlide?.content || '暂无内容' }}
              </div>
            </div>
          </div>
        </Transition>
      </Teleport>

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
