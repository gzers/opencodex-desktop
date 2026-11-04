/** jsdom has no native top layer; browser tests verify the real implementation. */
export function installNativeDialogStub() {
  Object.assign(HTMLDialogElement.prototype, {
    showModal(this: HTMLDialogElement) { this.setAttribute('open', '') },
    close(this: HTMLDialogElement) { this.removeAttribute('open') },
  })
}
