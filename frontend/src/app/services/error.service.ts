import { Service, signal } from '@angular/core';

@Service()
export class ErrorService {
    public currentError = signal<string | null>(null);

    setError(error: string) {
        this.currentError.set(error);
        setTimeout(() => {
            this.currentError.set(null);
        }, 2500);
    }
}
