import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Slide } from '../types/slide'
import type { StyleConfig } from '../types/style'
import type {
  PresentationSummary,
  PresentationDetail,
  UpdateSlideResponse,
} from '../types/api'
import { useApi } from '../composables/useApi'
import { useSse } from '../composables/useSse'

export const usePresentationStore = defineStore('presentation', () => {
  const api = useApi()

  const presentation = ref<PresentationDetail | null>(null)
  const presentations = ref<PresentationSummary[]>([])
  const selectedSid = ref<string | null>(null)
  const generatingSids = ref<Set<string>>(new Set())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const selectedSlide = computed(() => {
    if (!presentation.value || !selectedSid.value) return null
    return presentation.value.slides.find((s) => s.sid === selectedSid.value) ?? null
  })

  const currentImage = computed(() => {
    const slide = selectedSlide.value
    if (!slide || slide.images.length === 0) return null
    return slide.images[slide.activeImageIndex] ?? null
  })

  function selectSlide(sid: string) {
    selectedSid.value = sid
  }

  async function loadPresentations() {
    isLoading.value = true
    error.value = null
    try {
      const data = await api.get<{ presentations: PresentationSummary[] }>('')
      presentations.value = data.presentations
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load presentations'
    } finally {
      isLoading.value = false
    }
  }

  async function createPresentation(title: string) {
    error.value = null
    try {
      await api.post('', { title })
      await loadPresentations()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create presentation'
    }
  }

  async function loadPresentation(slug: string) {
    isLoading.value = true
    error.value = null
    try {
      const data = await api.get<PresentationDetail>(`/${slug}`)
      presentation.value = data
      if (data.slides.length > 0 && !selectedSid.value) {
        selectedSid.value = data.slides[0].sid
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load presentation'
    } finally {
      isLoading.value = false
    }
  }

  async function updateTitle(slug: string, title: string) {
    error.value = null
    try {
      await api.put(`/${slug}`, { title })
      if (presentation.value && presentation.value.slug === slug) {
        presentation.value.title = title
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update title'
    }
  }

  async function deletePresentation(slug: string) {
    error.value = null
    try {
      await api.del(`/${slug}`)
      presentations.value = presentations.value.filter((p) => p.slug !== slug)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete presentation'
    }
  }

  async function addSlide(slug: string, content: string, index?: number) {
    error.value = null
    try {
      const data = await api.post<{ slide: Slide }>(`/${slug}/slides`, { content, index })
      if (presentation.value) {
        const insertIndex = index ?? presentation.value.slides.length
        presentation.value.slides.splice(insertIndex, 0, data.slide)
        selectedSid.value = data.slide.sid
      }
      return data.slide
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add slide'
      return null
    }
  }

  async function updateSlide(
    slug: string,
    sid: string,
    body: { content?: string; activeImageIndex?: number },
  ): Promise<UpdateSlideResponse | null> {
    error.value = null
    try {
      const data = await api.put<UpdateSlideResponse>(`/${slug}/slides/${sid}`, body)
      if (presentation.value) {
        const idx = presentation.value.slides.findIndex((s) => s.sid === sid)
        if (idx !== -1) {
          presentation.value.slides[idx] = data.slide
        }
      }
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update slide'
      return null
    }
  }

  async function deleteSlide(slug: string, sid: string) {
    error.value = null
    try {
      await api.del(`/${slug}/slides/${sid}`)
      if (presentation.value) {
        presentation.value.slides = presentation.value.slides.filter((s) => s.sid !== sid)
        if (selectedSid.value === sid) {
          selectedSid.value = presentation.value.slides[0]?.sid ?? null
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete slide'
    }
  }

  async function reorderSlides(slug: string, orderedSids: string[]) {
    error.value = null
    try {
      const data = await api.put<{ slides: Slide[] }>(`/${slug}/slides`, { orderedSids })
      if (presentation.value) {
        presentation.value.slides = data.slides
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to reorder slides'
    }
  }

  function generateImage(slug: string, sid: string) {
    if (generatingSids.value.has(sid)) return
    generatingSids.value.add(sid)

    const sse = useSse()
    sse.on('complete', (data: unknown) => {
      const result = data as { image?: Slide['images'][0] }
      if (presentation.value && result.image) {
        const idx = presentation.value.slides.findIndex((s) => s.sid === sid)
        if (idx !== -1) {
          presentation.value.slides[idx].images.push(result.image)
          presentation.value.slides[idx].activeImageIndex =
            presentation.value.slides[idx].images.length - 1
        }
      }
      generatingSids.value.delete(sid)
      sse.close()
    })

    sse.on('progress', () => {})

    sse.on('error', (data: unknown) => {
      const result = data as { error?: string }
      console.error('Generation error:', result.error)
      generatingSids.value.delete(sid)
      sse.close()
    })

    sse.connect(`/api/ppt/${slug}/generate/${sid}`)
  }

  async function batchGenerate(slug: string, sids: string[]) {
    for (const sid of sids) {
      generatingSids.value.add(sid)
    }

    const sse = useSse()
    sse.on('complete', (data: unknown) => {
      const result = data as { sid: string; image?: Slide['images'][0] }
      if (presentation.value && result.image) {
        const idx = presentation.value.slides.findIndex((s) => s.sid === result.sid)
        if (idx !== -1) {
          presentation.value.slides[idx].images.push(result.image)
          presentation.value.slides[idx].activeImageIndex =
            presentation.value.slides[idx].images.length - 1
        }
      }
      generatingSids.value.delete(result.sid)
    })

    sse.on('done', () => {
      sse.close()
    })

    sse.on('error', (data: unknown) => {
      const result = data as { sid?: string; error?: string }
      if (result.sid) {
        generatingSids.value.delete(result.sid)
      }
      console.error('Batch generation error:', result.error)
    })

    sse.connect(`/api/ppt/${slug}/generate/batch`)
  }

  async function generateStyleCandidates(slug: string, prompt: string) {
    error.value = null
    try {
      const data = await api.post<{ candidates: string[]; prompt: string; errors?: string[] }>(
        `/${slug}/generate/style`,
        { prompt },
      )
      if (data.errors && data.errors.length > 0 && data.candidates.length === 0) {
        error.value = data.errors.join('\n')
      }
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to generate style candidates'
      return null
    }
  }

  async function selectStyle(slug: string, referenceImage: string) {
    error.value = null
    try {
      const data = await api.put<{ style: StyleConfig }>(`/${slug}/generate/style`, {
        referenceImage,
      })
      if (presentation.value) {
        presentation.value.style = data.style
      }
      return data.style
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to select style'
      return null
    }
  }

  async function cleanupCandidates(slug: string) {
    try {
      await api.del(`/${slug}/generate/style`)
      if (presentation.value) {
        presentation.value.style.candidates = []
      }
    } catch (e) {
      console.error('Failed to cleanup candidates:', e)
    }
  }

  function imageUrl(slug: string, filename: string) {
    return `/api/ppt/${slug}/images/${filename}`
  }

  return {
    presentation,
    presentations,
    selectedSid,
    selectedSlide,
    currentImage,
    generatingSids,
    isLoading,
    error,
    selectSlide,
    loadPresentations,
    createPresentation,
    loadPresentation,
    updateTitle,
    deletePresentation,
    addSlide,
    updateSlide,
    deleteSlide,
    reorderSlides,
    generateImage,
    batchGenerate,
    generateStyleCandidates,
    selectStyle,
    cleanupCandidates,
    imageUrl,
  }
})
