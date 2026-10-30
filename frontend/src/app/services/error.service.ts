import { Service, signal } from '@angular/core';

@Service()
export class ErrorService {
    private pendingErrors: string[] = [];
    public currentError = signal<string | null>(null);

    addError(error: string) {
        this.pendingErrors.push(error);
        if (this.pendingErrors.length == 1) {
            this.showError();
        }
    }

    private showError() {
        if (this.pendingErrors.length > 0) {
            let firstError = this.pendingErrors[0];
            this.pendingErrors = this.pendingErrors.filter((v, i) => i > 0);

            this.currentError.set(firstError);

            setTimeout(() => {
                this.showError();
            }, 2500);
        } else {
            this.currentError.set(null);
        }
    }
}
