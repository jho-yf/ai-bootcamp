<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { Slide } from '../../types/slide'
import { usePresentationStore } from '../../stores/presentation'

const props = defineProps<{
  slides: Slide[]
  slug: string
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const presentationStore = usePresentationStore()

const isPlaying = ref(false)
const playingIndex = ref(0)
let timer: ReturnType<typeof setInterval> | null = null
const INTERVAL = 5000

const slidesWithImages = computed(() =>
  props.slides.filter((s) => s.images.length > 0),
)

const currentSlide = computed(() => slidesWithImages.value[playingIndex.value] ?? null)

const currentImageUrl = computed(() => {
  const slide = currentSlide.value
  if (!slide) return null
  const image = slide.images[slide.activeImageIndex]
  if (!image) return null
  return presentationStore.imageUrl(props.slug, image.filename)
})

function startPlayback() {
  if (slidesWithImages.value.length === 0) return
  isPlaying.value = true
  playingIndex.value = 0
  restartTimer()
}

function restartTimer() {
  if (timer) clearInterval(timer)
  timer = setInterval(() => {
    showNext()
  }, INTERVAL)
}

function showPrev() {
  playingIndex.value =
    (playingIndex.value - 1 + slidesWithImages.value.length) % slidesWithImages.value.length
}

function showNext() {
  playingIndex.value = (playingIndex.value + 1) % slidesWithImages.value.length
}

function stopPlayback() {
  isPlaying.value = false
  if (timer) {
    clearInterval(timer)
    timer = null
  }
  emit('close')
}

function onKeydown(e: KeyboardEvent) {
  if (!isPlaying.value) return
  if (e.key === 'Escape') {
    stopPlayback()
  } else if (e.key === 'ArrowLeft') {
    showPrev()
    restartTimer()
  } else if (e.key === 'ArrowRight') {
    showNext()
    restartTimer()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
  if (timer) clearInterval(timer)
})

watch(
  () => props.visible,
  (val) => {
    if (val) {
      startPlayback()
    } else if (isPlaying.value) {
      isPlaying.value = false
      if (timer) {
        clearInterval(timer)
        timer = null
      }
    }
  },
)
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
      <div
        v-if="isPlaying"
        class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/95 bg-[radial-gradient(circle_at_center,#1f2937_0%,#020617_70%)] p-4"
        @click="stopPlayback"
      >
        <img
          v-if="currentImageUrl"
          :src="currentImageUrl"
          class="h-full w-full object-contain drop-shadow-2xl"
        />

        <div class="absolute bottom-8 left-1/2 -translate-x-1/2 text-white/60 text-sm">
          {{ playingIndex + 1 }} / {{ slidesWithImages.length }}
        </div>

        <div class="absolute top-4 right-4 text-white/40 text-sm">
          ← → 切换，ESC 退出
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
