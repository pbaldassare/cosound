-- Italian versions of DB-driven public content.
alter table public.plans add column features_it text[] not null default '{}';
alter table public.daws add column description_it text not null default '';

update public.plans set features_it = array['Analisi delle librerie personali','Ricerca e selezione dei campioni','Creazione MIDI','Arrangiamento assistito','Inserimento dei materiali nella DAW','Chat intelligente e prompt vocali'] where code = 'base';
update public.plans set features_it = array['Tutte le funzioni Base','Gestione avanzata dei sintetizzatori','Sound design','Automazioni','Mixing assistito','Analisi tecnica del progetto'] where code = 'pro';
update public.plans set features_it = array['Tutte le funzioni Pro','Mastering assistito','Analisi avanzata del mix','Controllo loudness e True Peak','Catene di mastering','Verifica tecnica del master finale'] where code = 'premium';

update public.daws set description_it = 'Prima integrazione prevista. COSOUND creerà le tracce, inserirà i campioni e programmerà il MIDI nei tuoi progetti Logic su macOS.' where slug = 'logic-pro';
update public.daws set description_it = 'Integrazione con Session e Arrangement view, usando i tuoi rack, device e librerie di campioni.' where slug = 'ableton-live';
update public.daws set description_it = 'Integrazione con Channel rack, Piano roll e Playlist, usando i tuoi generatori e campioni installati.' where slug = 'fl-studio';
update public.daws set description_it = 'Assistenza a livello di sessione per arrangiamento, editing e preparazione del mix.' where slug = 'pro-tools';
