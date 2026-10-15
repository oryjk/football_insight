import { computed, reactive, ref, watch } from 'vue'
import type { SeatSwapCandidate, SeatSwapRequest } from '../../../types/seatSwap'
import type { TicketWatchRegion } from '../../../types/ticketWatch'
import { resolveSeatSwapRegionColorGroup } from '../../../utils/stadiumRegions'
import {
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  hasSeatSwapFormErrors,
  previousSeatSwapStep,
  toggleDesiredSeatRegion,
  validateSeatSwapForm,
  type SeatSwapFormErrors,
  type SeatSwapFormState,
  type SeatSwapSelectionStep,
} from '../helpers'

interface SeatSwapPublishFormProps {
  visible: boolean
  regions: TicketWatchRegion[]
  myRequest: SeatSwapRequest | null
  presetCandidate: SeatSwapCandidate | null
}

export function useSeatSwapPublishForm(
  props: SeatSwapPublishFormProps,
  onSubmit: (payload: { form: SeatSwapFormState; presetCandidate: SeatSwapCandidate | null }) => void,
) {
  const selectionStep = ref<SeatSwapSelectionStep>('select_current')
  const stagedCurrentRegionKey = ref('')
  const stagedDesiredSeats = ref<SeatSwapFormState['desired_seats']>([])
  const formErrors = ref<SeatSwapFormErrors>({})

  const form = reactive<SeatSwapFormState>({
    current_region_key: '',
    current_region_name: '',
    current_row: '',
    current_seat_no: '',
    wechat_id: '',
    phone_number: '',
    desired_seats: [],
  })

  const selectionSteps = computed<Array<{ step: SeatSwapSelectionStep; index: number; label: string }>>(() =>
    props.presetCandidate
      ? [
          { step: 'select_current', index: 1, label: '我的座位' },
          { step: 'ready_to_publish', index: 2, label: '确认换座' },
        ]
      : [
          { step: 'select_current', index: 1, label: '当前' },
          { step: 'select_desired', index: 2, label: '目标' },
          { step: 'ready_to_publish', index: 3, label: '联系' },
        ],
  )

  const selectionStepIndex = computed(() =>
    selectionSteps.value.find((item) => item.step === selectionStep.value)?.index || 1,
  )

  const sheetEyebrow = computed(() =>
    `${props.presetCandidate ? '定向换座 · ' : ''}第 ${selectionStepIndex.value} 步 / 共 ${selectionSteps.value.length} 步`,
  )

  const sheetTitle = computed(() => {
    if (props.presetCandidate) {
      return selectionStep.value === 'select_current' ? '填写我的座位并发起换座' : '补充联系方式并确认换座'
    }
    if (selectionStep.value === 'select_current') return '点选当前座位分区'
    if (selectionStep.value === 'select_desired') return '点选目标座位分区'
    return '补充联系方式并发布'
  })

  const sheetMapMode = computed<'select-current' | 'select-desired' | 'review'>(() => {
    if (selectionStep.value === 'select_current') return 'select-current'
    if (selectionStep.value === 'select_desired') return 'select-desired'
    return 'review'
  })

  const stagedCurrentRegionName = computed(() => findRegion(stagedCurrentRegionKey.value)?.block_name || '')
  const stagedDesiredKeys = computed<string[]>(() => stagedDesiredSeats.value.map((s) => s.region_key).filter(Boolean))
  const confirmedDesiredKeys = computed<string[]>(() => form.desired_seats.map((s) => s.region_key).filter(Boolean))

  const canConfirmCurrentSelection = computed(() =>
    canConfirmCurrentSeatRegion(
      stagedCurrentRegionKey.value,
      form.current_row,
      form.current_seat_no,
    ),
  )
  const canConfirmDesiredSelection = computed(() => canConfirmDesiredSeatRegions(stagedDesiredSeats.value))

  watch(
    () => props.visible,
    (visible) => {
      if (!visible) {
        return
      }

      if (props.presetCandidate) {
        resetFormForNewRequest()
        stagedDesiredSeats.value = [
          {
            region_key: props.presetCandidate.current_region_key,
            region_name: props.presetCandidate.current_region_name,
            desired_row: props.presetCandidate.current_row || '',
            desired_seat_no: props.presetCandidate.current_seat_no || '',
          },
        ]
        form.desired_seats = stagedDesiredSeats.value.map((seat) => ({ ...seat }))
        selectionStep.value = 'select_current'
        return
      }

      if (props.myRequest) {
        fillFormFromMine()
      } else {
        resetFormForNewRequest()
      }
    },
  )

  function regionKey(region: TicketWatchRegion): string {
    return region.block_key || region.block_name
  }

  function findRegion(key: string): TicketWatchRegion | undefined {
    if (!key) return undefined
    return props.regions.find((r) => regionKey(r) === key)
  }

  function regionTagClass(regionName: string): string {
    return `tag--region-${resolveSeatSwapRegionColorGroup(regionName)}`
  }

  function resetFormForNewRequest(): void {
    form.current_region_key = ''
    form.current_region_name = ''
    form.current_row = ''
    form.current_seat_no = ''
    form.wechat_id = ''
    form.phone_number = ''
    form.desired_seats = []
    formErrors.value = {}
    stagedCurrentRegionKey.value = ''
    stagedDesiredSeats.value = []
    selectionStep.value = 'select_current'
  }

  function fillFormFromMine(): void {
    const mine = props.myRequest
    if (!mine) {
      resetFormForNewRequest()
      return
    }
    form.current_region_key = mine.current_region_key
    form.current_region_name = mine.current_region_name
    form.current_row = mine.current_row
    form.current_seat_no = mine.current_seat_no
    form.wechat_id = mine.contact?.wechat_id || ''
    form.phone_number = mine.contact?.phone_number || ''
    form.desired_seats = mine.desired_seats.map((s) => ({
      region_key: s.region_key,
      region_name: s.region_name,
      desired_row: s.desired_row || '',
      desired_seat_no: s.desired_seat_no || '',
    }))
    stagedCurrentRegionKey.value = mine.current_region_key
    stagedDesiredSeats.value = form.desired_seats.map((s) => ({ ...s }))
    formErrors.value = {}
    selectionStep.value = 'ready_to_publish'
  }

  function handleSheetMapTap(key: string): void {
    const region = findRegion(key)
    if (!region) return
    formErrors.value = {}
    if (selectionStep.value === 'select_current') {
      stagedCurrentRegionKey.value = key
    } else if (selectionStep.value === 'select_desired') {
      if (key === form.current_region_key) return
      stagedDesiredSeats.value = toggleDesiredSeatRegion(stagedDesiredSeats.value, {
        region_key: key,
        region_name: region.block_name,
      })
    }
  }

  function confirmCurrentSelection(): void {
    const errors: SeatSwapFormErrors = {}
    if (!stagedCurrentRegionKey.value.trim()) {
      errors.current_region_key = '请选择当前座位分区'
    }
    if (!form.current_row.trim()) {
      errors.current_row = '请输入当前排号'
    }
    if (!form.current_seat_no.trim()) {
      errors.current_seat_no = '请输入当前座号'
    }
    if (hasSeatSwapFormErrors(errors)) {
      formErrors.value = errors
      return
    }

    const region = findRegion(stagedCurrentRegionKey.value)
    if (!region) {
      formErrors.value = { current_region_key: '请选择当前座位分区' }
      return
    }
    form.current_region_key = regionKey(region)
    form.current_region_name = region.block_name
    if (props.presetCandidate && stagedDesiredSeats.value.length) {
      form.desired_seats = stagedDesiredSeats.value.map((s) => ({ ...s }))
      selectionStep.value = 'ready_to_publish'
      return
    }
    selectionStep.value = 'select_desired'
  }

  function confirmDesiredSelection(): void {
    if (!canConfirmDesiredSeatRegions(stagedDesiredSeats.value)) {
      formErrors.value = { desired_seats: '请选择目标座位分区' }
      return
    }
    form.desired_seats = stagedDesiredSeats.value.map((s) => ({ ...s }))
    selectionStep.value = 'ready_to_publish'
  }

  function goPreviousSelectionStep(): void {
    if (selectionStep.value === 'ready_to_publish') {
      stagedDesiredSeats.value = form.desired_seats.map((s) => ({ ...s }))
    }
    selectionStep.value = previousSeatSwapStep(selectionStep.value, Boolean(props.presetCandidate))
  }

  function jumpToStep(target: SeatSwapSelectionStep): void {
    const order = selectionSteps.value.map((item) => item.step)
    const currentIdx = order.indexOf(selectionStep.value)
    const targetIdx = order.indexOf(target)
    if (targetIdx < 0 || targetIdx >= currentIdx) return
    if (target === 'select_current') {
      stagedCurrentRegionKey.value = form.current_region_key || stagedCurrentRegionKey.value
    } else if (target === 'select_desired') {
      stagedDesiredSeats.value = form.desired_seats.map((s) => ({ ...s }))
    }
    selectionStep.value = target
  }

  function submit(): void {
    const errors = validateSeatSwapForm(form)
    formErrors.value = errors
    if (hasSeatSwapFormErrors(errors)) return

    onSubmit({
      form: {
        current_region_key: form.current_region_key,
        current_region_name: form.current_region_name,
        current_row: form.current_row,
        current_seat_no: form.current_seat_no,
        wechat_id: form.wechat_id,
        phone_number: form.phone_number,
        desired_seats: form.desired_seats.map((s) => ({ ...s })),
      },
      presetCandidate: props.presetCandidate,
    })
  }
  return {
    selectionStep, stagedCurrentRegionKey, stagedDesiredSeats, formErrors, form,
    selectionSteps, selectionStepIndex, sheetEyebrow, sheetTitle, sheetMapMode,
    stagedCurrentRegionName, stagedDesiredKeys, confirmedDesiredKeys,
    canConfirmCurrentSelection, canConfirmDesiredSelection, regionTagClass,
    handleSheetMapTap, confirmCurrentSelection, confirmDesiredSelection,
    goPreviousSelectionStep, jumpToStep, submit,
  }
}
