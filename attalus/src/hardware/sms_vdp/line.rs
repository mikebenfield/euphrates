use super::*;

pub fn line<V>(vdp: &mut V) -> Result<(), SmsVdpGraphicsError>
where
    V: SmsVdpInternal + SmsVdpGraphics,
{
    vdp.draw_line()?;

    let v = vdp.v();

    if v == vdp.active_lines() {
        // YYY - it's not completely clear to me whether this is the right
        // line on which to trigger a frame interrupt.
        let flags = vdp.status_flags();
        vdp.set_status_flags(flags | FRAME_INTERRUPT_FLAG);
        // XXX
        // manifests::SET_FRAME_INTERRUPT.send(vdp, Payload::U16([v, 0, 0, 0]));
    }

    let new_v = (v + 1) % vdp.total_lines();

    vdp.set_v(new_v);

    if new_v <= vdp.active_lines() {
        // yes, according to VDPTEST.sms, this really should be <=
        let line_counter = vdp.line_counter();
        vdp.set_line_counter(line_counter.wrapping_sub(1));
        if vdp.line_counter() == 0xFF {
            let reg_line_counter = vdp.reg_line_counter();
            vdp.set_line_counter(reg_line_counter);
            vdp.set_line_interrupt_pending(true);
            // XXX
            // manifests::SET_LINE_INTERRUPT
            //     .send(vdp, Payload::U16([v, reg_line_counter as u16, 0, 0]));
        }
    } else {
        let reg_line_counter = vdp.reg_line_counter();
        vdp.set_line_counter(reg_line_counter);
    }

    if new_v == 0 {
        let reg9 = unsafe { vdp.register_unchecked(9) };
        vdp.set_y_scroll(reg9);
    }

    let cycles = vdp.cycles();
    vdp.set_cycles(cycles + 342);

    return Ok(());
}
