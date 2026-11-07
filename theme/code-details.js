// Expand folded book content for print, then restore its prior open/closed state.
(() => {
    const closedForPrint = new Set();

    window.addEventListener('beforeprint', () => {
        const selector = ['code-details', 'quiz-answers', 'misconception-details', 'summary-details', 'reference-details', 'exercise-details', 'verification-details']
            .map(name => `details.${name}:not([open])`).join(', ');
        document.querySelectorAll(selector).forEach(details => {
            closedForPrint.add(details);
            details.open = true;
        });
    });

    window.addEventListener('afterprint', () => {
        closedForPrint.forEach(details => { details.open = false; });
        closedForPrint.clear();
    });
})();
