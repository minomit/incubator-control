// Contiene tutte le traduzioni
const translations = {
    it: {
        'page-title': "Gestore Incubate - La Tua App per l'Incubazione",
        'main-title': '🐣 Gestore Incubate Miste',
        'subtitle': 'La soluzione semplice per gestire incubate con specie e tempi diversi.',
        'screenshot-alt': "Screenshot dell'applicazione Gestore Incubate",
        'what-is-it-title': "Cos'è?",
        'what-is-it-p': "Questa applicazione ti aiuta a gestire le tue incubate, specialmente quando hai uova di specie diverse (galline, anatre, oche, quaglie) che richiedono tempi di incubazione differenti. L'app ti dice esattamente quando inserire ogni lotto di uova per farle schiudere tutte insieme!",
        'features-title': 'Funzionalità Principali',
        'feature-1': '✅ Gestione di incubate miste con più specie.',
        'feature-2': '✅ Calcolo automatico del giorno di inserimento per ogni lotto.',
        'feature-3': '✅ Interfaccia semplice e intuitiva in Italiano e Inglese.',
        'feature-4': "✅ Barra di progresso per vedere a che punto è l'incubata.",
        'feature-5': '✅ Promemoria visivi per i giorni di azione.',
        'download-title': 'Download',
        'download-p': "Scarica l'ultima versione per il tuo sistema operativo.",
        'download-button': 'Vai alla pagina dei Rilasci',
        'footer-text': '© 2024 minomitrugno - Rilasciato sotto licenza MIT'
    },
    en: {
        'page-title': 'Hatch Manager - Your Incubation App',
        'main-title': '🐣 Mixed Hatch Manager',
        'subtitle': 'The simple solution for managing hatches with different species and timings.',
        'screenshot-alt': 'Screenshot of the Hatch Manager application',
        'what-is-it-title': 'What is it?',
        'what-is-it-p': 'This application helps you manage your hatches, especially when you have eggs from different species (chickens, ducks, geese, quails) that require different incubation times. The app tells you exactly when to place each batch of eggs to have them all hatch together!',
        'features-title': 'Main Features',
        'feature-1': '✅ Manage mixed hatches with multiple species.',
        'feature-2': '✅ Automatic calculation of the setting day for each batch.',
        'feature-3': '✅ Simple and intuitive interface in Italian and English.',
        'feature-4': '✅ Progress bar to see the status of the hatch.',
        'feature-5': '✅ Visual reminders for action days.',
        'download-title': 'Download',
        'download-p': 'Download the latest version for your operating system.',
        'download-button': 'Go to the Releases Page',
        'footer-text': '© 2024 minomitrugno - Released under the MIT license'
    }
};

// Funzione per impostare la lingua
function setLanguage(lang) {
    // Aggiorna tutti gli elementi con l'attributo data-key
    document.querySelectorAll('[data-key]').forEach(element => {
        const key = element.getAttribute('data-key');
        // Gestisce anche l'attributo 'alt' per le immagini
        if (key.endsWith('-alt')) {
            element.setAttribute('alt', translations[lang][key]);
        } else {
            element.innerHTML = translations[lang][key];
        }
    });

    // Aggiorna l'attributo lang del tag <html> per accessibilità e SEO
    document.documentElement.lang = lang;

    // Salva la preferenza della lingua nel localStorage
    localStorage.setItem('language', lang);
}

// Funzione per caricare la lingua all'avvio della pagina
function loadLanguage() {
    // Controlla se una lingua è già stata salvata nel localStorage
    const savedLang = localStorage.getItem('language');
    
    // Controlla la lingua del browser come fallback
    const browserLang = navigator.language.substring(0, 2);

    // Determina quale lingua usare: salvata > browser > default (italiano)
    const langToSet = savedLang || (translations[browserLang] ? browserLang : 'it');
    
    setLanguage(langToSet);
}

// Esegui la funzione loadLanguage quando la pagina è pronta
document.addEventListener('DOMContentLoaded', loadLanguage);