import { describe, expect, test } from 'bun:test'

import { validateSeatSwapForm, type SeatSwapFormState } from './helpers'

function validForm(): SeatSwapFormState {
  return {
    current_region_key: 'A',
    current_region_name: 'A区',
    current_row: '8',
    current_seat_no: '15',
    wechat_id: 'wx-test',
    phone_number: '',
    desired_seats: [
      {
        region_key: 'B',
        region_name: 'B区',
        desired_row: '',
        desired_seat_no: '',
      },
    ],
  }
}

describe('seat swap form validation', () => {
  test('requires at least one contact method', () => {
    const errors = validateSeatSwapForm({
      ...validForm(),
      wechat_id: '',
      phone_number: '',
    })

    expect(errors.contact).toBe('请至少填写微信号或手机号')
  })

  test('requires current row and seat number', () => {
    const errors = validateSeatSwapForm({
      ...validForm(),
      current_row: '',
      current_seat_no: '',
    })

    expect(errors.current_row).toBe('请输入当前排号')
    expect(errors.current_seat_no).toBe('请输入当前座号')
  })

  test('requires desired region for each target', () => {
    const errors = validateSeatSwapForm({
      ...validForm(),
      desired_seats: [{ region_key: '', region_name: '', desired_row: '', desired_seat_no: '' }],
    })

    expect(errors.desired_seats).toBe('请选择想换到的分区')
  })
})
