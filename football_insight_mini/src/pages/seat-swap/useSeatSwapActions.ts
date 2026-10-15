import type { Ref } from 'vue'
import {
  MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID,
  cancelSeatSwapCandidateConfirmation,
  confirmSeatSwapCandidate,
  deleteMySeatSwapRequest,
  upsertMySeatSwapRequest,
  cancelMatchedSeatSwap,
} from '../../api/seatSwap'
import type { SeatSwapCandidate, SeatSwapCurrentResponse } from '../../types/seatSwap'
import { extractApiErrorMessage } from '../../utils/apiError'
import { filterOutMySeatSwapRequest, type SeatSwapFormState } from './helpers'

interface SeatSwapActionDependencies {
  submitting: Ref<boolean>
  currentView: Ref<SeatSwapCurrentResponse | null>
  publishSheetVisible: Ref<boolean>
  pendingConfirmTarget: Ref<SeatSwapCandidate | null>
  manageSheetVisible: Ref<boolean>
  miniProgramNoticeEnabled: Ref<boolean>
  pendingMatchedCancelTargetId: Ref<string>
  cancelReason: Ref<string>
  loadPage: () => Promise<void>
  openPublishSheetForCandidate: (candidate: SeatSwapCandidate) => void
}

export function useSeatSwapActions({
  submitting,
  currentView,
  publishSheetVisible,
  pendingConfirmTarget,
  manageSheetVisible,
  miniProgramNoticeEnabled,
  pendingMatchedCancelTargetId,
  cancelReason,
  loadPage,
  openPublishSheetForCandidate,
}: SeatSwapActionDependencies) {
  async function submitForm(payload: { form: SeatSwapFormState; presetCandidate: SeatSwapCandidate | null }): Promise<void> {
    const form = payload.form
    submitting.value = true
    try {
      const subscribeAccepted = await requestSeatSwapSubscribeMessage()
      await upsertMySeatSwapRequest({
        current_region_key: form.current_region_key,
        current_region_name: form.current_region_name,
        current_row: form.current_row,
        current_seat_no: form.current_seat_no,
        wechat_id: form.wechat_id || null,
        phone_number: form.phone_number || null,
        seat_swap_notice_enabled: subscribeAccepted,
        desired_seats: form.desired_seats.map((s) => ({
          region_key: s.region_key,
          region_name: s.region_name,
          desired_row: s.desired_row || null,
          desired_seat_no: s.desired_seat_no || null,
        })),
      })
      if (payload.presetCandidate) {
        await confirmSeatSwapCandidate(payload.presetCandidate.request_id)
        uni.showToast({ title: '已确认换座', icon: 'success' })
      } else {
        uni.showToast({ title: '发布成功', icon: 'success' })
      }
      publishSheetVisible.value = false
      pendingConfirmTarget.value = null
      await loadPage()
    } catch (err) {
      uni.showToast({ title: extractApiErrorMessage(err, '发布失败'), icon: 'none' })
    } finally {
      submitting.value = false
    }
  }

  async function deleteRequest(): Promise<void> {
    try {
      manageSheetVisible.value = false
      await deleteMySeatSwapRequest()
      if (currentView.value) {
        currentView.value = {
          ...currentView.value,
          my_request: null,
          candidates: filterOutMySeatSwapRequest(
            currentView.value.candidates,
            currentView.value.my_request?.request_id,
          ),
        }
      }
      miniProgramNoticeEnabled.value = false
      uni.showToast({ title: '已撤销', icon: 'success' })
      await loadPage()
    } catch (err) {
      manageSheetVisible.value = true
      uni.showToast({ title: extractApiErrorMessage(err, '撤销失败'), icon: 'none' })
    }
  }

  async function confirmCandidate(requestId: string): Promise<void> {
    const mine = currentView.value?.my_request
    if (!mine) {
      const candidate = currentView.value?.candidates.find((item) => item.request_id === requestId)
      if (!candidate) {
        uni.showToast({ title: '换座对象不存在', icon: 'none' })
        return
      }
      openPublishSheetForCandidate(candidate)
      return
    }

    try {
      await confirmSeatSwapCandidate(requestId)
      uni.showToast({ title: '已确认', icon: 'success' })
      await loadPage()
    } catch (err) {
      uni.showToast({ title: extractApiErrorMessage(err, '确认失败'), icon: 'none' })
    }
  }

  async function confirmSeatSwapCancellation(): Promise<boolean> {
    return await new Promise<boolean>((resolve) => {
      uni.showModal({
        title: '确认取消匹配',
        content: '取消后需要重新发起确认，确定继续吗？',
        confirmText: '确定取消',
        cancelText: '先保留',
        success: (result) => resolve(Boolean(result.confirm)),
        fail: () => resolve(false),
      })
    })
  }

  async function cancelCandidateConfirmation(requestId: string): Promise<void> {
    const confirmed = await confirmSeatSwapCancellation()
    if (!confirmed) {
      return
    }

    try {
      await cancelSeatSwapCandidateConfirmation(requestId)
      uni.showToast({ title: '已取消匹配', icon: 'success' })
      await loadPage()
    } catch (err) {
      uni.showToast({ title: extractApiErrorMessage(err, '取消失败'), icon: 'none' })
    }
  }

  async function requestSeatSwapSubscribeMessage(): Promise<boolean> {
    if (!MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID) {
      return miniProgramNoticeEnabled.value
    }

    // #ifdef MP-WEIXIN
    return await new Promise<boolean>((resolve) => {
      uni.requestSubscribeMessage({
        tmplIds: [MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID],
        success: (result) => {
          const subscribeResult = result as unknown as Record<string, string | undefined>
          const accepted = subscribeResult[MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID] === 'accept'
          miniProgramNoticeEnabled.value = accepted
          resolve(accepted)
        },
        fail: () => {
          resolve(miniProgramNoticeEnabled.value)
        },
      })
    })
    // #endif

    return miniProgramNoticeEnabled.value
  }

  async function submitMatchedCancel(): Promise<void> {
    const matchedId = currentView.value?.my_request?.request_id
    const target = currentView.value?.candidates.find((candidate) => {
      if (pendingMatchedCancelTargetId.value) {
        return candidate.request_id === pendingMatchedCancelTargetId.value
      }
      return candidate.status === 'matched'
    })
    if (!matchedId || !target) {
      uni.showToast({ title: '暂无可撤销的匹配', icon: 'none' })
      return
    }
    if (!cancelReason.value.trim()) {
      uni.showToast({ title: '请填写撤销说明', icon: 'none' })
      return
    }
    try {
      await cancelMatchedSeatSwap(target.request_id, {
        reason: cancelReason.value,
        evidence_file_name: '',
        evidence_content_type: '',
        evidence_base64: '',
      })
      uni.showToast({ title: '已提交撤销', icon: 'success' })
      manageSheetVisible.value = false
      pendingMatchedCancelTargetId.value = ''
      await loadPage()
    } catch (err) {
      uni.showToast({ title: extractApiErrorMessage(err, '提交撤销失败'), icon: 'none' })
    }
  }

  return { submitForm, deleteRequest, confirmCandidate, cancelCandidateConfirmation, submitMatchedCancel }
}
