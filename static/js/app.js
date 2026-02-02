// Theme toggle
document.addEventListener('DOMContentLoaded', () => {
    const toggle = document.getElementById('theme-toggle');
    const html = document.documentElement;

    // Load saved theme
    const savedTheme = localStorage.getItem('theme') || 'light';
    html.setAttribute('data-theme', savedTheme);

    if (toggle) {
        toggle.addEventListener('click', () => {
            const current = html.getAttribute('data-theme');
            const next = current === 'light' ? 'dark' : 'light';
            html.setAttribute('data-theme', next);
            localStorage.setItem('theme', next);
        });
    }

    // Character counter for compose
    const textarea = document.querySelector('.compose-box textarea');
    const counter = document.querySelector('.char-count');

    if (textarea && counter) {
        textarea.addEventListener('input', () => {
            const remaining = 280 - textarea.value.length;
            counter.textContent = remaining;
            counter.style.color = remaining < 20 ? 'var(--error)' : 'var(--text-secondary)';
        });
    }
});

// Auto-resize textarea
document.addEventListener('input', (e) => {
    if (e.target.matches('textarea')) {
        e.target.style.height = 'auto';
        e.target.style.height = e.target.scrollHeight + 'px';
    }
});
