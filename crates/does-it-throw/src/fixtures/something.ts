export const someThrow = () => {
    throw new Error('never gonna let you down');
}

export function CallToThrow () {
    someThrow()
} 