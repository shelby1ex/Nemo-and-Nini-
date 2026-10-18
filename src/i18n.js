/* =============================================================================
   LANGUAGES  —  src/i18n.js
   -----------------------------------------------------------------------------
   Changing the language used to only move a value into a file. This makes it
   reach the window.

   Translations are keyed by the **English source text** rather than by some
   made-up id. That has one big practical advantage: nothing in the markup has
   to be annotated, so a sentence can be reworded once and every language still
   lines up. The cost is that the source text is the contract, so the English
   strings in the pages must stay in step with the keys below — `missing()`
   below will tell us if they drift.

   Only languages we can say are complete are offered. A language that silently
   half-applies is worse than one that isn't there at all, which is exactly the
   complaint this fixes.
   ========================================================================== */

/* What we ship. `rtl` marks the languages written right-to-left. */
export const LANGUAGES = [
  { code: 'en', label: 'English', rtl: false },
  { code: 'fr', label: 'Français', rtl: false },
  { code: 'ar', label: 'العربية', rtl: true },
];

/** Is this a code we can actually switch to? Anything else falls back to English. */
export function known(code) {
  const s = String(code == null ? '' : code).toLowerCase();
  return LANGUAGES.some(l => l.code === s) ? s : 'en';
}

export function isRTL(code) {
  const l = LANGUAGES.find(x => x.code === known(code));
  return !!(l && l.rtl);
}

/* ── the dictionaries ───────────────────────────────────────────────────── */
/* French and Arabic, keyed by the exact English string on the page. */

const fr = {
  /* window frame */
  'Minimise': 'Réduire',
  'Close': 'Fermer',
  'Settings': 'Réglages',

  /* rail */
  'AI engine': "Moteur d'IA",
  'General': 'Général',
  'Companions': 'Compagnons',
  'Desktop & taskbar': 'Bureau et barre des tâches',
  'Appearance': 'Apparence',
  'Advanced': 'Avancé',

  /* page headings */
  'opencode runs locally. Everything stays on this machine.':
    "opencode s'exécute localement. Tout reste sur cette machine.",
  'Startup, sound and the language they speak.': 'Démarrage, son et langue parlée.',
  'Nemo and Nini on your real desktop.': 'Nemo et Nini sur votre vrai bureau.',
  'How the overlay talks to your real windows.': "Comment la surcouche parle à vos vraies fenêtres.",
  'The island and the two of them.': "L'île et les deux compagnons.",
  'Logs, folders and the big reset.': 'Journaux, dossiers et réinitialisation complète.',

  /* AI engine */
  'Engine': 'Moteur',
  'opencode': 'opencode',
  'The local engine that thinks. Started automatically.':
    'Le moteur local qui réfléchit. Démarré automatiquement.',
  'Start': 'Démarrer',
  'Stop': 'Arrêter',
  'Restart': 'Redémarrer',
  'Engine running': 'Moteur en marche',
  'Engine stopped': 'Moteur arrêté',
  'Working…': 'Travail en cours…',
  'Not started yet.': 'Pas encore démarré.',
  'running · ': 'en marche · ',
  'stopped': 'arrêté',
  'error': 'erreur',

  'Provider keys': 'Clés des fournisseurs',
  'Connected only': 'Connectés seulement',
  'Show every provider opencode knows about.':
    'Afficher tous les fournisseurs connus par opencode.',
  'Search providers': 'Rechercher un fournisseur',
  'connected': 'connecté',
  'no key': 'pas de clé',
  'API key': 'Clé API',
  'Save': 'Enregistrer',
  'Clear': 'Effacer',
  'Start the engine to list providers.': 'Démarrez le moteur pour lister les fournisseurs.',
  'Loading providers…': 'Chargement des fournisseurs…',
  'No provider matches.': 'Aucun fournisseur ne correspond.',
  'Could not reach the engine: ': 'Impossible de joindre le moteur : ',
  'Paste a key first.': "Collez d'abord une clé.",
  ' key removed.': ' supprimée.',
  ' connected.': ' connecté.',

  'Model': 'Modèle',
  'Free models only': 'Modèles gratuits seulement',
  'Every model listed here costs nothing to run.':
    'Chaque modèle listé ici ne coûte rien à exécuter.',
  'Refresh': 'Actualiser',
  'Model saved.': 'Modèle enregistré.',
  'Default': 'Par défaut',
  'Search models': 'Rechercher un modèle',
  'Choose a free model…': 'Choisir un modèle gratuit…',
  'No free model matches.': 'Aucun modèle gratuit ne correspond.',
  'No free models found. Start the engine and refresh.':
    'Aucun modèle gratuit trouvé. Démarrez le moteur et actualisez.',
  'no free models': 'aucun modèle gratuit',
  'Loading free models…': 'Chargement des modèles gratuits…',
  'Search across ': 'Rechercher parmi ',
  ' free models': ' modèles gratuits',

  /* General */
  'Startup & sound': 'Démarrage et son',
  'Open at login': 'Ouvrir à la connexion',
  'Start Nemo & Nini when you sign in to Windows.':
    'Lancer Nemo & Nini quand vous vous connectez à Windows.',
  'Sound': 'Son',
  'Little voices, slaps and reactions.': 'Petites voix, claques et réactions.',
  'Volume': 'Volume',
  'How loud they get.': 'Leur niveau sonore.',
  'Language': 'Langue',
  'For their speech and the interface, where translated.':
    "Pour leurs paroles et l'interface, là où c'est traduit.",
  'English': 'Anglais',
  'Français': 'Français',
  'العربية': 'العربية',

  /* Companions */
  'On the desktop': 'Sur le bureau',
  'Everyone out at launch': 'Tous dehors au lancement',
  'Send them to the desktop the moment the app starts.':
    'Les envoyer sur le bureau dès que l’application démarre.',
  'Size': 'Taille',
  'How big they are.': 'Leur taille.',
  'Walk speed': 'Vitesse de marche',
  'How fast they cross your screen.': 'À quelle vitesse ils traversent votre écran.',

  /* Desktop & taskbar */
  'Real windows': 'Vraies fenêtres',
  'Click through': 'Traverser les clics',
  'Let clicks pass through the overlay except on the island or them.':
    'Laisser les clics passer à travers la surcouche, sauf sur l’île ou eux.',
  'Sit on your windows': "S'asseoir sur vos fenêtres",
  'The island rests on top of your app windows like a shelf.':
    "L'île se pose au-dessus de vos fenêtres comme une étagère.",
  'Floor offset': 'Décalage du sol',
  'How far up from the taskbar they stand.': "À quelle hauteur ils se tiennent au-dessus de la barre des tâches.",

  /* Appearance */
  'Island': 'Île',
  'Distance from the top': 'Distance depuis le haut',
  'Where the island parks, measured from the top of the screen.':
    "Où l'île se gare, mesuré depuis le haut de l'écran.",
  'Island opacity': 'Opacité de l’île',
  'How solid the island looks.': 'À quel point l’île est opaque.',
  'Companions': 'Compagnons',
  'Companion opacity': 'Opacité des compagnons',
  'How solid they look.': 'À quel point ils sont opaques.',
  'Dark mode': 'Mode sombre',
  'Light, dark, or follow Windows.': 'Clair, sombre, ou suivre Windows.',
  'Light': 'Clair',
  'Dark': 'Sombre',
  'System': 'Système',
  'Theme': 'Thème',
  'The look of the island and of Settings.': "L'apparence de l'île et des réglages.",
  'Themes': 'Thèmes',

  /* Advanced */
  'Engine log': 'Journal du moteur',
  'Folders': 'Dossiers',
  'Settings & engine data': 'Réglages et données du moteur',
  'Where this window writes its file.': 'Où cette fenêtre écrit son fichier.',
  'Logs': 'Journaux',
  'Engine stderr and app logs.': "Erreur standard du moteur et journaux de l'application.",
  'Open folder': 'Ouvrir le dossier',
  'About': 'À propos',
  'App': 'Application',
  'Reset everything': 'Tout réinitialiser',
  'Puts every setting back to its default. Nothing else is touched.':
    'Remet chaque réglage à sa valeur par défaut. Rien d’autre n’est touché.',
  'Reset': 'Réinitialiser',
  'Everything is back to default.': 'Tout est revenu aux valeurs par défaut.',
  'File picker unavailable.': 'Sélecteur de fichier indisponible.',
  'Saved.': 'Enregistré.',
  'Refreshed.': 'Actualisé.',
  'Could not save: ': 'Impossible d’enregistrer : ',
  'Engine stopped.': 'Moteur arrêté.',
  'Engine started.': 'Moteur démarré.',
  'Engine restarted.': 'Moteur redémarré.',

  /* engine update card */
  'Engine update': 'Mise à jour du moteur',
  'Installed version': 'Version installée',
  'Latest version': 'Dernière version',
  'Check for updates': 'Vérifier les mises à jour',
  'Download': 'Télécharger',
  'Update engine': 'Mettre à jour le moteur',
  'Up to date': 'À jour',
  'Checking…': 'Recherche de mises à jour…',
  'Downloading…': 'Téléchargement…',
  'Verifying…': 'Vérification du fichier…',
  'Installing…': 'Installation…',
  'Restarting the engine…': 'Redémarrage du moteur…',
  'Engine updated.': 'Moteur mis à jour.',
  'Update failed.': 'La mise à jour a échoué.',
  'Update failed — rolled back to the previous version.':
    'La mise à jour a échoué — retour à la version précédente.',
  'The opencode runtime is not installed yet. Press Download to get it.':
    "Le moteur opencode n'est pas encore installé. Appuyez sur Télécharger pour l'obtenir.",
  'A new version is available.': 'Une nouvelle version est disponible.',
  "The runtime lives in the app's own data folder, never in your projects.":
    "Le moteur vit dans le dossier de données de l'application, jamais dans vos projets.",
  'Leave empty for the copy the app manages. Point it at a binary only to override.':
    "Laissez vide pour la copie gérée par l'application. Indiquez un binaire uniquement pour remplacer.",
  'allow': 'autoriser',
  'ask': 'demander',
  'deny': 'refuser',

  /* the island's own labels (built in JS, never walked) */
  'Ask anything': "Demandez n'importe quoi",
  'Still answering…': 'Répond encore…',
  'Thinking…': 'Réflexion…',
  'Answering…': 'Réponse…',
  'Stopped.': 'Arrêté.',
  'Searching': 'Recherche',
  'Reading file': 'Lecture du fichier',
  'Running code': 'Exécution du code',
  'Editing': 'Édition',
  'Working': 'Travail en cours',
  'Ask them anything.': "Demandez-leur n'importe quoi.",
  'Everything the three of you say stays in this thread — it will still be here next time.':
    "Tout ce que vous dites reste dans ce fil — il sera encore là la prochaine fois.",
  'Activity': 'Activité',
  'Approve': 'Approuver',
  'Context': 'Contexte',
  'waiting for you': 'vous attend',
  'Ask the agent…': "Demandez à l'agent…",
  'Nothing yet. Ask something above — every line here comes from the engine.':
    "Rien pour l'instant. Posez une question ci-dessus — chaque ligne vient du moteur.",
  'No pending requests. When the engine needs a tool it asks here.':
    "Aucune demande en attente. Quand le moteur a besoin d'un outil, il demande ici.",
  'No windows open.': 'Aucune fenêtre ouverte.',
  'No context attached.': 'Aucun contexte joint.',
  'Permission': 'Permission',
  'Allow once': 'Autoriser une fois',
  'Always': 'Toujours',
  'Reject': 'Refuser',
  'Thinking': 'Réflexion',
  'Shell': 'Shell',
  'Edit': 'Édition',
  'WebFetch': 'WebFetch',
  'Search': 'Recherche',
  'Read': 'Lecture',
  'Task': 'Tâche',
  'Tool': 'Outil',
  'Danger': 'Dangereux',
  'The engine is not running.': 'Le moteur ne tourne pas.',
  'The engine refused that.': 'Le moteur a refusé.',
  'sent to the engine': 'envoyé au moteur',
  'by you': 'par vous',
  'running': 'en cours',
  'completed': 'terminé',

  /* companion states + emotes (rendered from JS tables) */
  'Idle': 'Inactif',
  'Done': 'Terminé',
  'Error': 'Erreur',
  'Needs your go-ahead': 'Attend votre feu vert',
  'Asks a question': 'Pose une question',
  'Rate limited': 'Limité par le débit',
  'Asleep': 'Endormi',
  'Dizzy': 'Étourdi',
  'Love': 'Amour',
  'Surprised': 'Surpris',
  'Proud': 'Fier',
  'Yawn': 'Bâillement',
  'Happy': 'Heureux',
  'Annoyed': 'Agacé',

  /* settings labels the harvest walk rewrites (never appeared as literals) */
  'Providers & API keys': 'Fournisseurs et clés API',
  'Mode': 'Mode',
  'Status': 'État',
  'Start with the app': 'Démarrer avec l’application',
  'Bring the engine up as soon as Nemo & Nini opens.':
    'Démarre le moteur dès que Nemo & Nini s’ouvre.',
  'Runtime': 'Binaire du moteur',
  'Port': 'Port',
  '0 picks a free port on every launch.': '0 = choisit un port libre à chaque lancement.',
  'Workspace': 'Dossier de travail',
  'The folder the engine works in. Empty = your home folder.':
    'Le dossier dans lequel travaille le moteur. Vide = votre dossier personnel.',
  'Default model': 'Modèle par défaut',
  'Agent': 'Agent',
  "opencode's default agent.": "L’agent par défaut d’opencode.",
  'Connected providers': 'Fournisseurs connectés',
  "Keys are handed straight to the engine — they are never written to the app's settings file.":
    'Les clés sont remises directement au moteur — elles ne sont jamais écrites dans le fichier de réglages de l’application.',
  'Nemo and Nini start loose on your desktop instead of waiting in the island.':
    'Nemo et Nini démarrent libres sur votre bureau au lieu d’attendre dans l’îlot.',
  'How big they are on screen.': 'Leur taille à l’écran.',
  'Nemo runs when he wants; this is the general pace.':
    'Nemo court quand il veut ; ceci est le rythme général.',
  'Let every click reach your apps. Turn this off and the overlay swallows them — useful for testing only.':
    'Laisse chaque clic atteindre vos applications. Désactivez-le et l’overlay les avale — utile uniquement pour les tests.',
  'They climb onto the title bar of whatever is open and ride along when it moves.':
    'Ils grimpent sur la barre de titre de la fenêtre ouverte et la suivent quand elle bouge.',
  'Nudge the line they walk on, relative to the top of the real taskbar.':
    'Ajustez la ligne qu’ils parcourent, par rapport au haut de la barre des tâches réelle.',
  'Dark or light': 'Sombre ou clair',
  'How far the island hangs from the top edge of the screen.':
    'À quelle distance l’îlot pend du bord supérieur de l’écran.',
  'Opacity': 'Opacité',
  'Make it blend into the wallpaper.': 'Le fondre dans le fond d’écran.',
  'How solid Nemo and Nini look on the desktop.':
    'La densité de Nemo et Nini sur le bureau.',
  'The look of the island and of Settings. Pick one and both change together.':
    'L’apparence de l’îlot et des Réglages. Choisissez-en un, les deux changent ensemble.',
  'Browse…': 'Parcourir…',
};

const ar = {
  /* window frame */
  'Minimise': 'تصغير',
  'Close': 'إغلاق',
  'Settings': 'الإعدادات',

  /* rail */
  'AI engine': 'محرّك الذكاء الاصطناعي',
  'General': 'عام',
  'Companions': 'الرفاق',
  'Desktop & taskbar': 'سطح المكتب وشريط المهام',
  'Appearance': 'المظهر',
  'Advanced': 'متقدّم',

  /* page headings */
  'opencode runs locally. Everything stays on this machine.':
    'يعمل opencode محليًا. كل شيء يبقى على هذا الجهاز.',
  'Startup, sound and the language they speak.': 'التشغيل والصوت واللغة التي يتحدثون بها.',
  'Nemo and Nini on your real desktop.': 'نيمو ونيني على سطح مكتبك الحقيقي.',
  'How the overlay talks to your real windows.': 'كيف تتحدّث الطبقة العلوية إلى نوافذك الحقيقية.',
  'The island and the two of them.': 'الجزيرة والاثنان معًا.',
  'Logs, folders and the big reset.': 'السجلات والمجلدات وإعادة التعيين الكاملة.',

  /* AI engine */
  'Engine': 'المحرّك',
  'opencode': 'opencode',
  'The local engine that thinks. Started automatically.':
    'المحرّك المحلي الذي يفكّر. يبدأ تلقائيًا.',
  'Start': 'تشغيل',
  'Stop': 'إيقاف',
  'Restart': 'إعادة تشغيل',
  'Engine running': 'المحرّك يعمل',
  'Engine stopped': 'المحرّك متوقّف',
  'Working…': 'جارٍ العمل…',
  'Not started yet.': 'لم يبدأ بعد.',
  'running · ': 'يعمل · ',
  'stopped': 'متوقّف',
  'error': 'خطأ',

  'Provider keys': 'مفاتيح المزوّدين',
  'Connected only': 'المتصلون فقط',
  'Show every provider opencode knows about.':
    'إظهار كل المزوّدين الذين يعرفهم opencode.',
  'Search providers': 'ابحث عن مزوّد',
  'connected': 'متصل',
  'no key': 'بلا مفتاح',
  'API key': 'مفتاح الواجهة',
  'Save': 'حفظ',
  'Clear': 'مسح',
  'Start the engine to list providers.': 'شغّل المحرّك لعرض المزوّدين.',
  'Loading providers…': 'جارٍ تحميل المزوّدين…',
  'No provider matches.': 'لا يوجد مزوّد مطابق.',
  'Could not reach the engine: ': 'تعذّر الوصول إلى المحرّك: ',
  'Paste a key first.': 'الصق مفتاحًا أولًا.',
  ' key removed.': ' تم حذف المفتاح.',
  ' connected.': ' متصل.',

  'Model': 'النموذج',
  'Free models only': 'النماذج المجانية فقط',
  'Every model listed here costs nothing to run.':
    'كل نموذج مُدرج هنا لا يكلّف شيئًا عند تشغيله.',
  'Refresh': 'تحديث',
  'Model saved.': 'تم حفظ النموذج.',
  'Default': 'الافتراضي',
  'Search models': 'ابحث عن نموذج',
  'Choose a free model…': 'اختر نموذجًا مجانيًا…',
  'No free model matches.': 'لا يوجد نموذج مجاني مطابق.',
  'No free models found. Start the engine and refresh.':
    'لم يُعثر على نماذج مجانية. شغّل المحرّك ثم حدّث.',
  'no free models': 'لا نماذج مجانية',
  'Loading free models…': 'جارٍ تحميل النماذج المجانية…',
  'Search across ': 'ابحث في ',
  ' free models': ' نموذجًا مجانيًا',

  /* General */
  'Startup & sound': 'التشغيل والصوت',
  'Open at login': 'الفتح عند تسجيل الدخول',
  'Start Nemo & Nini when you sign in to Windows.':
    'تشغيل Nemo & Nini عند تسجيل دخولك إلى ويندوز.',
  'Sound': 'الصوت',
  'Little voices, slaps and reactions.': 'أصوات صغيرة وصفقات وردود فعل.',
  'Volume': 'مستوى الصوت',
  'How loud they get.': 'شدة صوتهم.',
  'Language': 'اللغة',
  'For their speech and the interface, where translated.':
    'لغة كلامهم والواجهة، حيثما كانت مترجمة.',
  'English': 'الإنجليزية',
  'Français': 'الفرنسية',
  'العربية': 'العربية',

  /* Companions */
  'On the desktop': 'على سطح المكتب',
  'Everyone out at launch': 'الجميع للخارج عند التشغيل',
  'Send them to the desktop the moment the app starts.':
    'أرسلهم إلى سطح المكتب فور بدء التطبيق.',
  'Size': 'الحجم',
  'How big they are.': 'حجمهم.',
  'Walk speed': 'سرعة المشي',
  'How fast they cross your screen.': 'سرعة عبورهم لشاشتك.',

  /* Desktop & taskbar */
  'Real windows': 'النوافذ الحقيقية',
  'Click through': 'النقر الشفاف',
  'Let clicks pass through the overlay except on the island or them.':
    'دع النقرات تمرّ عبر الطبقة العلوية إلا فوق الجزيرة أو عليهم.',
  'Sit on your windows': 'اجلس فوق نوافذك',
  'The island rests on top of your app windows like a shelf.':
    'تستقر الجزيرة فوق نوافذ تطبيقاتك كرفّة.',
  'Floor offset': 'إزاحة الأرضية',
  'How far up from the taskbar they stand.': 'كم يرتفعون فوق شريط المهام.',

  /* Appearance */
  'Island': 'الجزيرة',
  'Distance from the top': 'المسافة من الأعلى',
  'Where the island parks, measured from the top of the screen.':
    'أين ترتكب الجزيرة، تُقاس من أعلى الشاشة.',
  'Island opacity': 'عتامة الجزيرة',
  'How solid the island looks.': 'درجة صلابة الجزيرة.',
  'Companions': 'الرفاق',
  'Companion opacity': 'عتامة الرفاق',
  'How solid they look.': 'درجة صلابتهم.',
  'Dark mode': 'الوضع الداكن',
  'Light, dark, or follow Windows.': 'فاتح أو داكن أو يتبع ويندوز.',
  'Light': 'فاتح',
  'Dark': 'داكن',
  'System': 'النظام',
  'Theme': 'السمة',
  'The look of the island and of Settings.': 'مظهر الجزيرة والإعدادات.',
  'Themes': 'السمات',

  /* Advanced */
  'Engine log': 'سجلّ المحرّك',
  'Folders': 'المجلدات',
  'Settings & engine data': 'الإعدادات وبيانات المحرّك',
  'Where this window writes its file.': 'أين تكتب هذه النافذة ملفها.',
  'Logs': 'السجلات',
  'Engine stderr and app logs.': 'مخرجات المحرّك وسجلات التطبيق.',
  'Open folder': 'فتح المجلد',
  'About': 'حول',
  'App': 'التطبيق',
  'Reset everything': 'إعادة تعيين كل شيء',
  'Puts every setting back to its default. Nothing else is touched.':
    'يعيد كل إعداد إلى قيمته الافتراضية. لا يُمسّ شيء آخر.',
  'Reset': 'إعادة تعيين',
  'Everything is back to default.': 'كل شيء عاد إلى الافتراضي.',
  'File picker unavailable.': 'منتقي الملفات غير متاح.',
  'Saved.': 'تم الحفظ.',
  'Refreshed.': 'تم التحديث.',
  'Could not save: ': 'تعذّر الحفظ: ',
  'Engine stopped.': 'تم إيقاف المحرّك.',
  'Engine started.': 'تم تشغيل المحرّك.',
  'Engine restarted.': 'تمت إعادة تشغيل المحرّك.',

  /* engine update card */
  'Engine update': 'تحديث المحرّك',
  'Installed version': 'النسخة المثبتة',
  'Latest version': 'أحدث نسخة',
  'Check for updates': 'التحقق من التحديثات',
  'Download': 'تنزيل',
  'Update engine': 'تحديث المحرّك',
  'Up to date': 'محدَّث',
  'Checking…': 'جارٍ البحث عن تحديثات…',
  'Downloading…': 'جارٍ التنزيل…',
  'Verifying…': 'جارٍ التحقق من الملف…',
  'Installing…': 'جارٍ التثبيت…',
  'Restarting the engine…': 'جارٍ إعادة تشغيل المحرّك…',
  'Engine updated.': 'تم تحديث المحرّك.',
  'Update failed.': 'فشل التحديث.',
  'Update failed — rolled back to the previous version.':
    'فشل التحديث — تم الرجوع إلى النسخة السابقة.',
  'The opencode runtime is not installed yet. Press Download to get it.':
    'لم يُثبَّت محرّك opencode بعد. اضغط «تنزيل» للحصول عليه.',
  'A new version is available.': 'تتوفر نسخة جديدة.',
  "The runtime lives in the app's own data folder, never in your projects.":
    'يقيم المحرّك في مجلد بيانات التطبيق، وليس في مشاريعك أبدًا.',
  'Leave empty for the copy the app manages. Point it at a binary only to override.':
    'اتركه فارغًا للنسخة التي تديرها التطبيق. حدِّد ملفًا تنفيذيًا فقط للاستبدال.',
  'allow': 'السماح',
  'ask': 'السؤال',
  'deny': 'المنع',

  /* the island's own labels (built in JS, never walked) */
  'Ask anything': 'اسأل أي شيء',
  'Still answering…': 'ما زال يجيب…',
  'Thinking…': 'يفكّر…',
  'Answering…': 'يجيب…',
  'Stopped.': 'تم الإيقاف.',
  'Searching': 'يبحث',
  'Reading file': 'يقرأ ملفًا',
  'Running code': 'يشغّل كودًا',
  'Editing': 'يحرّر',
  'Working': 'يعمل',
  'Ask them anything.': 'اسألهما أي شيء.',
  'Everything the three of you say stays in this thread — it will still be here next time.':
    'كل ما تقولونه يبقى في هذا المحادثة — سيكون هنا في المرة القادمة أيضًا.',
  'Activity': 'النشاط',
  'Approve': 'الموافقات',
  'Context': 'السياق',
  'waiting for you': 'ينتظر ردّك',
  'Ask the agent…': 'اسأل الوكيل…',
  'Nothing yet. Ask something above — every line here comes from the engine.':
    'لا شيء بعد. اسأل شيئًا أعلاه — كل سطر هنا من المحرّك.',
  'No pending requests. When the engine needs a tool it asks here.':
    'لا طلبات معلّقة. عندما يحتاج المحرّك إلى أداة يطلبها هنا.',
  'No windows open.': 'لا نوافذ مفتوحة.',
  'No context attached.': 'لا سياق مرفق.',
  'Permission': 'إذن',
  'Allow once': 'السماح مرة واحدة',
  'Always': 'دائمًا',
  'Reject': 'رفض',
  'Thinking': 'يفكّر',
  'Shell': 'طرفية',
  'Edit': 'تحرير',
  'WebFetch': 'جلب الويب',
  'Search': 'بحث',
  'Read': 'قراءة',
  'Task': 'مهمة',
  'Tool': 'أداة',
  'Danger': 'خطِر',
  'The engine is not running.': 'المحرّك غير يعمل.',
  'The engine refused that.': 'رفض المحرّك ذلك.',
  'sent to the engine': 'أُرسل إلى المحرّك',
  'by you': 'من قبلك',
  'running': 'قيد التشغيل',
  'completed': 'اكتمل',

  /* companion states + emotes (rendered from JS tables) */
  'Idle': 'خامل',
  'Done': 'تم',
  'Error': 'خطأ',
  'Needs your go-ahead': 'ينتظر موافقتك',
  'Asks a question': 'يطرح سؤالًا',
  'Rate limited': 'محدودية المعدّل',
  'Asleep': 'نائم',
  'Dizzy': 'مدوّخ',
  'Love': 'حب',
  'Surprised': 'متفاجئ',
  'Proud': 'فخور',
  'Yawn': 'تأفّف',
  'Happy': 'سعيد',
  'Annoyed': 'منزعج',

  /* settings labels the harvest walk rewrites (never appeared as literals) */
  'Providers & API keys': 'المزوّدون ومفاتيح API',
  'Mode': 'الوضع',
  'Status': 'الحالة',
  'Start with the app': 'التشغيل مع التطبيق',
  'Bring the engine up as soon as Nemo & Nini opens.':
    'يشغّل المحرّك فور فتح Nemo و Nini.',
  'Runtime': 'الملف التشغيلي للمحرّك',
  'Port': 'المنفذ',
  '0 picks a free port on every launch.': '0 يختار منفذًا حرًا في كل تشغيل.',
  'Workspace': 'مجلد العمل',
  'The folder the engine works in. Empty = your home folder.':
    'المجلد الذي يعمل فيه المحرّك. فارغ = مجلدك الشخصي.',
  'Default model': 'النموذج الافتراضي',
  'Agent': 'الوكيل',
  "opencode's default agent.": 'الوكيل الافتراضي لـ opencode.',
  'Connected providers': 'المزوّدون المتصلون',
  "Keys are handed straight to the engine — they are never written to the app's settings file.":
    'تُسلَّم المفاتيح مباشرة إلى المحرّك — ولا تُكتب أبدًا في ملف إعدادات التطبيق.',
  'Nemo and Nini start loose on your desktop instead of waiting in the island.':
    'يبدأ Nemo و Nini حرّين على سطح مكتبك بدل انتظارهما في الجزيرة.',
  'How big they are on screen.': 'حجمهما على الشاشة.',
  'Nemo runs when he wants; this is the general pace.':
    'يجري Nemo متى شاء؛ هذا هو الإيقاع العام.',
  'Let every click reach your apps. Turn this off and the overlay swallows them — useful for testing only.':
    'يسمح لكل نقرة بالوصول إلى تطبيقاتك. أطفئه ويلتهم الطبقة العلوية النقرات — مفيد للاختبار فقط.',
  'They climb onto the title bar of whatever is open and ride along when it moves.':
    'يتسلقان شريط العنوان لأي نافذة مفتوحة ويتبعانه عند حركته.',
  'Nudge the line they walk on, relative to the top of the real taskbar.':
    'عدّل الخط الذي يمشيان عليه نسبةً إلى أعلى شريط المهام الحقيقي.',
  'Dark or light': 'داكن أو فاتح',
  'How far the island hangs from the top edge of the screen.':
    'ما مدى تبعُّد الجزيرة عن الحافة العليا للشاشة.',
  'Opacity': 'الشفافية',
  'Make it blend into the wallpaper.': 'لدمجه مع خلفية سطح المكتب.',
  'How solid Nemo and Nini look on the desktop.':
    'كثافة ظهور Nemo و Nini على سطح المكتب.',
  'The look of the island and of Settings. Pick one and both change together.':
    'مظهر الجزيرة والإعدادات. اختر أحدهما يتغيّر الاثنان معًا.',
  'Browse…': 'تصفّح…',
};

const DICT = { en: {}, fr, ar };

/* ── translating ────────────────────────────────────────────────────────── */

let lang = 'en';

/** One string, in the current language — falling back to English, then to itself. */
export function t(text) {
  const s = String(text == null ? '' : text);
  if (lang === 'en') return s;
  const d = DICT[lang];
  return (d && d[s]) || s;
}

/** How many English keys no language covers — a drift alarm, not an error. */
export function missing() {
  const en = Object.keys(DICT.fr);
  const arKeys = new Set(Object.keys(DICT.ar));
  return en.filter(k => !arKeys.has(k)).length;
}

/*
   Walking the page.

   A node is translatable if it is a leaf (all its content is its own text), or
   if it keeps its icons and only its trailing text (which is how the rail's
   buttons are built: an <svg>, then a label). Anything else is left alone —
   blanking an element's markup to translate it would take its icons with it.
*/
const SELECTOR = [
  '.nav-item', '.nav-foot > span',
  '.titlebar .nm span',
  '.head h1', '.head p',
  '.card > h2', '.row-t > b', '.row-t > span', '.note',
  '.btn', '.pill', '.kv > span', '.kv > b', 'option', '.val',
  '.seg button',
].join(',');

/* The English text each translated node started with. Stored once, on first
   pass, so switching language repeatedly is a lookup rather than a guess. */
let sources = null;

function harvest() {
  sources = [];
  for (const el of document.querySelectorAll(SELECTOR)) {
    let node = null;
    if (!el.children.length && el.textContent.trim()) node = el;
    else {
      const last = el.lastChild;
      if (last && last.nodeType === 3 && last.nodeValue.trim()) node = last;
    }
    if (!node) continue;
    const src = node.nodeValue === undefined ? node.textContent : node.nodeValue;
    sources.push({ node, src: src.replace(/\s+/g, ' ').trim() });
  }
}

/** Paint every harvested string into the current language. */
export function apply() {
  if (!sources) harvest();
  for (const s of sources) {
    const out = t(s.src);
    if (s.node.nodeValue === undefined) s.node.textContent = out;
    else s.node.nodeValue = out;
  }
}

/** Point the document at a language: text, direction, and the <html lang>. */
export function set(code) {
  lang = known(code);
  const root = document.documentElement;
  root.lang = lang;
  /* Only pages that opt in get their markup rewritten. The island builds most
     of its text in JS and owns those labels itself, so walking it would risk
     writing a stale translation over a label that has since changed. It still
     gets the language, so any string it renders can go through `t()`. */
  if (root.dataset.i18nDom === '1') {
    root.dir = isRTL(lang) ? 'rtl' : 'ltr';
    apply();
  }
  try {
    window.dispatchEvent(new CustomEvent('nn:lang', { detail: { lang } }));
  } catch (_) { /* no DOM */ }
}

export function current() {
  return lang;
}
