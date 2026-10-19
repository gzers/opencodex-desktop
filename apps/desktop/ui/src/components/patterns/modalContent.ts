/** Presentation fields only. Resolution and focus remain owned by AppModal. */
export interface ModalField {key:string;label:string;placeholder?:string;value?:string;type?:'text'|'password'}
export interface ModalContentSpec {title:string;body:string;confirmLabel?:string;primaryKind?:'primary'|'danger';cancelLabel?:string;deleteLabel?:string;resolveLabel?:string;wide?:boolean;fields?:ModalField[]}
