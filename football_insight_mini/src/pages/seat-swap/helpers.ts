export interface SeatSwapDesiredSeatFormState {
  region_key: string
  region_name: string
  desired_row: string
  desired_seat_no: string
}

export interface SeatSwapFormState {
  current_region_key: string
  current_region_name: string
  current_row: string
  current_seat_no: string
  wechat_id: string
  phone_number: string
  desired_seats: SeatSwapDesiredSeatFormState[]
}

export interface SeatSwapFormErrors {
  current_region_key?: string
  current_row?: string
  current_seat_no?: string
  desired_seats?: string
  contact?: string
  phone_number?: string
}

export function validateSeatSwapForm(form: SeatSwapFormState): SeatSwapFormErrors {
  const errors: SeatSwapFormErrors = {}

  if (!form.current_region_key.trim()) {
    errors.current_region_key = '请选择当前分区'
  }

  if (!form.current_row.trim()) {
    errors.current_row = '请输入当前排号'
  }

  if (!form.current_seat_no.trim()) {
    errors.current_seat_no = '请输入当前座号'
  }

  if (!form.desired_seats.length || form.desired_seats.some((seat) => !seat.region_key.trim())) {
    errors.desired_seats = '请选择想换到的分区'
  }

  const phone = form.phone_number.trim()
  const wechat = form.wechat_id.trim()
  if (!phone && !wechat) {
    errors.contact = '请至少填写微信号或手机号'
  }

  if (phone && !/^1\d{10}$/.test(phone)) {
    errors.phone_number = '请输入 11 位手机号'
  }

  return errors
}

export function hasSeatSwapFormErrors(errors: SeatSwapFormErrors): boolean {
  return Object.keys(errors).length > 0
}

export function formatSeatLabel(input: {
  current_region_name: string
  current_row: string
  current_seat_no: string
}): string {
  return `${input.current_region_name} ${input.current_row}排 ${input.current_seat_no}号`
}

export function statusLabel(status: string): string {
  switch (status) {
    case 'communicable':
      return '可沟通'
    case 'waiting_peer_confirmation':
      return '等待对方确认'
    case 'peer_confirmed_me':
      return '对方已确认你'
    case 'matched':
      return '已匹配成功'
    default:
      return '仅展示'
  }
}
