import db, { initDatabase } from './db';

interface Quote {
  username: string;
  content: string;
}

const quotes: Quote[] = [
  // Socrates (469-399 BC)
  { username: 'socrates', content: 'The only true wisdom is in knowing you know nothing.' },
  { username: 'socrates', content: 'An unexamined life is not worth living.' },
  { username: 'socrates', content: 'I cannot teach anybody anything. I can only make them think.' },
  { username: 'socrates', content: 'Be kind, for everyone you meet is fighting a hard battle.' },
  { username: 'socrates', content: 'Wonder is the beginning of wisdom.' },
  { username: 'socrates', content: 'To find yourself, think for yourself.' },
  { username: 'socrates', content: 'He who is not contented with what he has, would not be contented with what he would like to have.' },
  { username: 'socrates', content: 'True knowledge exists in knowing that you know nothing.' },
  { username: 'socrates', content: 'By all means, marry. If you get a good wife, you\'ll become happy; if you get a bad one, you\'ll become a philosopher.' },
  { username: 'socrates', content: 'The secret of change is to focus all of your energy not on fighting the old, but on building the new.' },
  { username: 'socrates', content: 'Beware the barrenness of a busy life.' },
  { username: 'socrates', content: 'Education is the kindling of a flame, not the filling of a vessel.' },

  // Plato (428-348 BC)
  { username: 'plato', content: 'The beginning is the most important part of the work.' },
  { username: 'plato', content: 'We can easily forgive a child who is afraid of the dark; the real tragedy of life is when men are afraid of the light.' },
  { username: 'plato', content: 'Wise men speak because they have something to say; fools because they have to say something.' },
  { username: 'plato', content: 'The first and greatest victory is to conquer yourself.' },
  { username: 'plato', content: 'Music gives a soul to the universe, wings to the mind, flight to the imagination and life to everything.' },
  { username: 'plato', content: 'Those who tell the stories rule society.' },
  { username: 'plato', content: 'Good people do not need laws to tell them to act responsibly, while bad people will find a way around the laws.' },
  { username: 'plato', content: 'Reality is created by the mind. We can change our reality by changing our mind.' },
  { username: 'plato', content: 'Opinion is the medium between knowledge and ignorance.' },
  { username: 'plato', content: 'The measure of a man is what he does with power.' },
  { username: 'plato', content: 'Never discourage anyone who continually makes progress, no matter how slow.' },
  { username: 'plato', content: 'Ignorance, the root and stem of every evil.' },

  // Aristotle (384-322 BC)
  { username: 'aristotle', content: 'We are what we repeatedly do. Excellence, then, is not an act, but a habit.' },
  { username: 'aristotle', content: 'It is the mark of an educated mind to be able to entertain a thought without accepting it.' },
  { username: 'aristotle', content: 'Knowing yourself is the beginning of all wisdom.' },
  { username: 'aristotle', content: 'The whole is greater than the sum of its parts.' },
  { username: 'aristotle', content: 'Quality is not an act, it is a habit.' },
  { username: 'aristotle', content: 'Patience is bitter, but its fruit is sweet.' },
  { username: 'aristotle', content: 'Happiness depends upon ourselves.' },
  { username: 'aristotle', content: 'The roots of education are bitter, but the fruit is sweet.' },
  { username: 'aristotle', content: 'The more you know, the more you realize you don\'t know.' },
  { username: 'aristotle', content: 'Hope is a waking dream.' },
  { username: 'aristotle', content: 'Pleasure in the job puts perfection in the work.' },
  { username: 'aristotle', content: 'To avoid criticism say nothing, do nothing, be nothing.' },

  // Marcus Aurelius (121-180 AD)
  { username: 'marcusaurelius', content: 'You have power over your mind - not outside events. Realize this, and you will find strength.' },
  { username: 'marcusaurelius', content: 'The happiness of your life depends upon the quality of your thoughts.' },
  { username: 'marcusaurelius', content: 'Waste no more time arguing about what a good man should be. Be one.' },
  { username: 'marcusaurelius', content: 'If you are distressed by anything external, the pain is not due to the thing itself, but to your estimate of it.' },
  { username: 'marcusaurelius', content: 'The best revenge is to be unlike him who performed the injury.' },
  { username: 'marcusaurelius', content: 'Very little is needed to make a happy life; it is all within yourself, in your way of thinking.' },
  { username: 'marcusaurelius', content: 'The universe is change; our life is what our thoughts make it.' },
  { username: 'marcusaurelius', content: 'When you arise in the morning, think of what a precious privilege it is to be alive.' },
  { username: 'marcusaurelius', content: 'Accept the things to which fate binds you, and love the people with whom fate brings you together.' },
  { username: 'marcusaurelius', content: 'The soul becomes dyed with the color of its thoughts.' },
  { username: 'marcusaurelius', content: 'Everything we hear is an opinion, not a fact. Everything we see is a perspective, not the truth.' },
  { username: 'marcusaurelius', content: 'It is not death that a man should fear, but he should fear never beginning to live.' },

  // Seneca (4 BC - 65 AD)
  { username: 'seneca', content: 'Luck is what happens when preparation meets opportunity.' },
  { username: 'seneca', content: 'We suffer more often in imagination than in reality.' },
  { username: 'seneca', content: 'Life is long if you know how to use it.' },
  { username: 'seneca', content: 'Difficulties strengthen the mind, as labor does the body.' },
  { username: 'seneca', content: 'Sometimes even to live is an act of courage.' },
  { username: 'seneca', content: 'It is not the man who has too little, but the man who craves more, that is poor.' },
  { username: 'seneca', content: 'Begin at once to live, and count each separate day as a separate life.' },
  { username: 'seneca', content: 'He who fears death will never do anything worth of a man who is alive.' },
  { username: 'seneca', content: 'As long as you live, keep learning how to live.' },
  { username: 'seneca', content: 'True happiness is to enjoy the present, without anxious dependence upon the future.' },
  { username: 'seneca', content: 'The whole future lies in uncertainty: live immediately.' },
  { username: 'seneca', content: 'Wherever there is a human being, there is an opportunity for a kindness.' },

  // Friedrich Nietzsche (1844-1900)
  { username: 'nietzsche', content: 'He who has a why to live can bear almost any how.' },
  { username: 'nietzsche', content: 'That which does not kill us makes us stronger.' },
  { username: 'nietzsche', content: 'Without music, life would be a mistake.' },
  { username: 'nietzsche', content: 'And those who were seen dancing were thought to be insane by those who could not hear the music.' },
  { username: 'nietzsche', content: 'There are no facts, only interpretations.' },
  { username: 'nietzsche', content: 'To live is to suffer, to survive is to find some meaning in the suffering.' },
  { username: 'nietzsche', content: 'The individual has always had to struggle to keep from being overwhelmed by the tribe.' },
  { username: 'nietzsche', content: 'In heaven, all the interesting people are missing.' },
  { username: 'nietzsche', content: 'The advantage of a bad memory is that one enjoys several times the same good things for the first time.' },
  { username: 'nietzsche', content: 'I\'m not upset that you lied to me, I\'m upset that from now on I can\'t believe you.' },
  { username: 'nietzsche', content: 'The snake which cannot cast its skin has to die. As well the minds which are prevented from changing their opinions.' },
  { username: 'nietzsche', content: 'There is always some madness in love. But there is also always some reason in madness.' },

  // Søren Kierkegaard (1813-1855)
  { username: 'kierkegaard', content: 'Life can only be understood backwards; but it must be lived forwards.' },
  { username: 'kierkegaard', content: 'Anxiety is the dizziness of freedom.' },
  { username: 'kierkegaard', content: 'The function of prayer is not to influence God, but rather to change the nature of the one who prays.' },
  { username: 'kierkegaard', content: 'People demand freedom of speech as a compensation for the freedom of thought which they seldom use.' },
  { username: 'kierkegaard', content: 'Once you label me you negate me.' },
  { username: 'kierkegaard', content: 'Most men pursue pleasure with such breathless haste that they hurry past it.' },
  { username: 'kierkegaard', content: 'Life is not a problem to be solved, but a reality to be experienced.' },
  { username: 'kierkegaard', content: 'The tyrant dies and his rule is over, the martyr dies and his rule begins.' },
  { username: 'kierkegaard', content: 'There are two ways to be fooled. One is to believe what isn\'t true; the other is to refuse to believe what is true.' },
  { username: 'kierkegaard', content: 'Boredom is the root of all evil - the despairing refusal to be oneself.' },
  { username: 'kierkegaard', content: 'The highest and most beautiful things in life are not to be heard about, nor read about, nor seen but, if one will, are to be lived.' },
  { username: 'kierkegaard', content: 'To dare is to lose one\'s footing momentarily. Not to dare is to lose oneself.' },

  // Jean-Paul Sartre (1905-1980)
  { username: 'sartre', content: 'Man is condemned to be free; because once thrown into the world, he is responsible for everything he does.' },
  { username: 'sartre', content: 'Hell is other people.' },
  { username: 'sartre', content: 'We are our choices.' },
  { username: 'sartre', content: 'Freedom is what you do with what\'s been done to you.' },
  { username: 'sartre', content: 'Everything has been figured out, except how to live.' },
  { username: 'sartre', content: 'Life begins on the other side of despair.' },
  { username: 'sartre', content: 'If you are lonely when you\'re alone, you are in bad company.' },
  { username: 'sartre', content: 'Man is nothing else but what he makes of himself.' },
  { username: 'sartre', content: 'To know what life is worth you have to risk it once in a while.' },
  { username: 'sartre', content: 'We do not know what we want and yet we are responsible for what we are.' },
  { username: 'sartre', content: 'Three o\'clock is always too late or too early for anything you want to do.' },
  { username: 'sartre', content: 'Only the guy who isn\'t rowing has time to rock the boat.' },

  // Albert Camus (1913-1960)
  { username: 'camus', content: 'In the depth of winter, I finally learned that within me there lay an invincible summer.' },
  { username: 'camus', content: 'The only way to deal with an unfree world is to become so absolutely free that your very existence is an act of rebellion.' },
  { username: 'camus', content: 'Don\'t walk in front of me; I may not follow. Don\'t walk behind me; I may not lead. Just walk beside me and be my friend.' },
  { username: 'camus', content: 'You will never be happy if you continue to search for what happiness consists of. You will never live if you are looking for the meaning of life.' },
  { username: 'camus', content: 'Real generosity towards the future lies in giving all to the present.' },
  { username: 'camus', content: 'Man is the only creature who refuses to be what he is.' },
  { username: 'camus', content: 'Nobody realizes that some people expend tremendous energy merely to be normal.' },
  { username: 'camus', content: 'Should I kill myself, or have a cup of coffee?' },
  { username: 'camus', content: 'The purpose of a writer is to keep civilization from destroying itself.' },
  { username: 'camus', content: 'Autumn is a second spring when every leaf is a flower.' },
  { username: 'camus', content: 'To be happy, we must not be too concerned with others.' },
  { username: 'camus', content: 'It is the job of thinking people not to be on the side of the executioners.' },

  // Fyodor Dostoevsky (1821-1881)
  { username: 'dostoevsky', content: 'The mystery of human existence lies not in just staying alive, but in finding something to live for.' },
  { username: 'dostoevsky', content: 'To live without Hope is to Cease to live.' },
  { username: 'dostoevsky', content: 'Man is sometimes extraordinarily, passionately, in love with suffering.' },
  { username: 'dostoevsky', content: 'The soul is healed by being with children.' },
  { username: 'dostoevsky', content: 'If you want to overcome the whole world, overcome yourself.' },
  { username: 'dostoevsky', content: 'The cleverest of all, in my opinion, is the man who calls himself a fool at least once a month.' },
  { username: 'dostoevsky', content: 'Pain and suffering are always inevitable for a large intelligence and a deep heart.' },
  { username: 'dostoevsky', content: 'Nothing is easier than to denounce the evildoer; nothing is more difficult than to understand him.' },
  { username: 'dostoevsky', content: 'Beauty will save the world.' },
  { username: 'dostoevsky', content: 'We sometimes encounter people, even perfect strangers, who begin to interest us at first sight.' },
  { username: 'dostoevsky', content: 'Above all, don\'t lie to yourself. The man who lies to himself and listens to his own lie comes to a point that he cannot distinguish the truth.' },
  { username: 'dostoevsky', content: 'To go wrong in one\'s own way is better than to go right in someone else\'s.' },

  // Leo Tolstoy (1828-1910)
  { username: 'tolstoy', content: 'Everyone thinks of changing the world, but no one thinks of changing himself.' },
  { username: 'tolstoy', content: 'If you want to be happy, be.' },
  { username: 'tolstoy', content: 'Wrong does not cease to be wrong because the majority share in it.' },
  { username: 'tolstoy', content: 'The two most powerful warriors are patience and time.' },
  { username: 'tolstoy', content: 'There is no greatness where there is not simplicity, goodness, and truth.' },
  { username: 'tolstoy', content: 'All happy families are alike; each unhappy family is unhappy in its own way.' },
  { username: 'tolstoy', content: 'Respect was invented to cover the empty place where love should be.' },
  { username: 'tolstoy', content: 'Spring is the time of plans and projects.' },
  { username: 'tolstoy', content: 'True life is lived when tiny changes occur.' },
  { username: 'tolstoy', content: 'It is amazing how complete is the delusion that beauty is goodness.' },
  { username: 'tolstoy', content: 'The strongest of all warriors are these two — Time and Patience.' },
  { username: 'tolstoy', content: 'To know God and to live is one and the same thing. God is Life.' },

  // Franz Kafka (1883-1924)
  { username: 'kafka', content: 'I am free and that is why I am lost.' },
  { username: 'kafka', content: 'A book must be the axe for the frozen sea within us.' },
  { username: 'kafka', content: 'Start with what is right rather than what is acceptable.' },
  { username: 'kafka', content: 'Paths are made by walking.' },
  { username: 'kafka', content: 'Better to have, and not need, than to need, and not have.' },
  { username: 'kafka', content: 'I am a cage, in search of a bird.' },
  { username: 'kafka', content: 'Don\'t bend; don\'t water it down; don\'t try to make it logical; don\'t edit your own soul according to the fashion.' },
  { username: 'kafka', content: 'By believing passionately in something that still does not exist, we create it. The nonexistent is whatever we have not sufficiently desired.' },
  { username: 'kafka', content: 'I think we ought to read only the kind of books that wound and stab us.' },
  { username: 'kafka', content: 'Anyone who keeps the ability to see beauty never grows old.' },
  { username: 'kafka', content: 'In the fight between you and the world, back the world.' },
  { username: 'kafka', content: 'Every revolution evaporates and leaves behind only the slime of a new bureaucracy.' },

  // William Shakespeare (1564-1616)
  { username: 'shakespeare', content: 'To be, or not to be, that is the question.' },
  { username: 'shakespeare', content: 'All the world\'s a stage, and all the men and women merely players.' },
  { username: 'shakespeare', content: 'The course of true love never did run smooth.' },
  { username: 'shakespeare', content: 'To thine own self be true.' },
  { username: 'shakespeare', content: 'Love all, trust a few, do wrong to none.' },
  { username: 'shakespeare', content: 'Some are born great, some achieve greatness, and some have greatness thrust upon them.' },
  { username: 'shakespeare', content: 'The fool doth think he is wise, but the wise man knows himself to be a fool.' },
  { username: 'shakespeare', content: 'Cowards die many times before their deaths; the valiant never taste of death but once.' },
  { username: 'shakespeare', content: 'What\'s in a name? That which we call a rose by any other name would smell as sweet.' },
  { username: 'shakespeare', content: 'We know what we are, but know not what we may be.' },
  { username: 'shakespeare', content: 'Hell is empty and all the devils are here.' },
  { username: 'shakespeare', content: 'The fault, dear Brutus, is not in our stars, but in ourselves.' },

  // Oscar Wilde (1854-1900)
  { username: 'oscarwilde', content: 'Be yourself; everyone else is already taken.' },
  { username: 'oscarwilde', content: 'To live is the rarest thing in the world. Most people exist, that is all.' },
  { username: 'oscarwilde', content: 'I can resist everything except temptation.' },
  { username: 'oscarwilde', content: 'Always forgive your enemies; nothing annoys them so much.' },
  { username: 'oscarwilde', content: 'Experience is merely the name men gave to their mistakes.' },
  { username: 'oscarwilde', content: 'The truth is rarely pure and never simple.' },
  { username: 'oscarwilde', content: 'Man is least himself when he talks in his own person. Give him a mask, and he will tell you the truth.' },
  { username: 'oscarwilde', content: 'We are all in the gutter, but some of us are looking at the stars.' },
  { username: 'oscarwilde', content: 'I have the simplest tastes. I am always satisfied with the best.' },
  { username: 'oscarwilde', content: 'The only way to get rid of temptation is to yield to it.' },
  { username: 'oscarwilde', content: 'A cynic is a man who knows the price of everything and the value of nothing.' },
  { username: 'oscarwilde', content: 'I don\'t want to go to heaven. None of my friends are there.' },

  // Virginia Woolf (1882-1941)
  { username: 'virginiawoolf', content: 'You cannot find peace by avoiding life.' },
  { username: 'virginiawoolf', content: 'For most of history, Anonymous was a woman.' },
  { username: 'virginiawoolf', content: 'A woman must have money and a room of her own if she is to write fiction.' },
  { username: 'virginiawoolf', content: 'Arrange whatever pieces come your way.' },
  { username: 'virginiawoolf', content: 'As a woman I have no country. As a woman I want no country. As a woman, my country is the whole world.' },
  { username: 'virginiawoolf', content: 'Books are the mirrors of the soul.' },
  { username: 'virginiawoolf', content: 'If you do not tell the truth about yourself you cannot tell it about other people.' },
  { username: 'virginiawoolf', content: 'The beauty of the world has two edges, one of laughter, one of anguish, cutting the heart asunder.' },
  { username: 'virginiawoolf', content: 'Lock up your libraries if you like; but there is no gate, no lock, no bolt that you can set upon the freedom of my mind.' },
  { username: 'virginiawoolf', content: 'One cannot think well, love well, sleep well, if one has not dined well.' },
  { username: 'virginiawoolf', content: 'Nothing thicker than a knife\'s blade separates happiness from melancholy.' },
  { username: 'virginiawoolf', content: 'Words are the wildest, freest, most irresponsible, most unteachable of all things.' },

  // Simone de Beauvoir (1908-1986)
  { username: 'simonedebeauvoir', content: 'One is not born, but rather becomes, a woman.' },
  { username: 'simonedebeauvoir', content: 'Change your life today. Don\'t gamble on the future, act now, without delay.' },
  { username: 'simonedebeauvoir', content: 'I am too intelligent, too demanding, and too resourceful for anyone to be able to take charge of me entirely.' },
  { username: 'simonedebeauvoir', content: 'All oppression creates a state of war.' },
  { username: 'simonedebeauvoir', content: 'Few tasks are more like the torture of Sisyphus than housework, with its endless repetition.' },
  { username: 'simonedebeauvoir', content: 'To catch a husband is an art; to hold him is a job.' },
  { username: 'simonedebeauvoir', content: 'Man is defined as a human being and a woman as a female — whenever she behaves as a human being she is said to imitate the male.' },
  { username: 'simonedebeauvoir', content: 'One\'s life has value so long as one attributes value to the life of others, by means of love, friendship, indignation and compassion.' },
  { username: 'simonedebeauvoir', content: 'The body is not a thing, it is a situation: it is our grasp on the world and our sketch of our project.' },
  { username: 'simonedebeauvoir', content: 'I am incapable of conceiving infinity, and yet I do not accept finity.' },
  { username: 'simonedebeauvoir', content: 'If you live long enough, you\'ll see that every victory turns into a defeat.' },
  { username: 'simonedebeauvoir', content: 'The day knowledge was preferred to wisdom, manipulation had triumphed over purpose.' },

  // Hannah Arendt (1906-1975)
  { username: 'hannaharendt', content: 'The sad truth is that most evil is done by people who never make up their minds to be good or evil.' },
  { username: 'hannaharendt', content: 'Forgiveness is the key to action and freedom.' },
  { username: 'hannaharendt', content: 'The most radical revolutionary will become a conservative the day after the revolution.' },
  { username: 'hannaharendt', content: 'No one has the right to obey.' },
  { username: 'hannaharendt', content: 'Thinking does not bring knowledge as do the sciences. Thinking does not produce usable practical wisdom. Thinking does not solve the riddles of the universe.' },
  { username: 'hannaharendt', content: 'The ideal subject of totalitarian rule is not the convinced Nazi or the convinced Communist, but people for whom the distinction between fact and fiction no longer exists.' },
  { username: 'hannaharendt', content: 'The greatest enemy of authority is contempt, and the surest way to undermine it is laughter.' },
  { username: 'hannaharendt', content: 'Courage is indispensable because in politics not life but the world is at stake.' },
  { username: 'hannaharendt', content: 'The trouble with lying and deceiving is that their efficiency depends entirely upon a clear notion of the truth that the liar and deceiver wishes to hide.' },
  { username: 'hannaharendt', content: 'Nothing we use or hear or touch can be expressed in words that equal what is given by the senses.' },
  { username: 'hannaharendt', content: 'Power and violence are opposites; where the one rules absolutely, the other is absent.' },
  { username: 'hannaharendt', content: 'Education is the point at which we decide whether we love the world enough to assume responsibility for it.' },

  // Confucius (551-479 BC)
  { username: 'confucius', content: 'It does not matter how slowly you go as long as you do not stop.' },
  { username: 'confucius', content: 'Our greatest glory is not in never falling, but in rising every time we fall.' },
  { username: 'confucius', content: 'The man who moves a mountain begins by carrying away small stones.' },
  { username: 'confucius', content: 'Everything has beauty, but not everyone sees it.' },
  { username: 'confucius', content: 'Choose a job you love, and you will never have to work a day in your life.' },
  { username: 'confucius', content: 'Silence is a true friend who never betrays.' },
  { username: 'confucius', content: 'To see what is right and not do it is want of courage.' },
  { username: 'confucius', content: 'The superior man is modest in his speech, but exceeds in his actions.' },
  { username: 'confucius', content: 'When anger rises, think of the consequences.' },
  { username: 'confucius', content: 'Before you embark on a journey of revenge, dig two graves.' },
  { username: 'confucius', content: 'Real knowledge is to know the extent of one\'s ignorance.' },
  { username: 'confucius', content: 'Study the past if you would define the future.' },

  // Lao Tzu (6th century BC)
  { username: 'laotzu', content: 'A journey of a thousand miles begins with a single step.' },
  { username: 'laotzu', content: 'When I let go of what I am, I become what I might be.' },
  { username: 'laotzu', content: 'Nature does not hurry, yet everything is accomplished.' },
  { username: 'laotzu', content: 'He who knows, does not speak. He who speaks, does not know.' },
  { username: 'laotzu', content: 'Knowing others is intelligence; knowing yourself is true wisdom. Mastering others is strength; mastering yourself is true power.' },
  { username: 'laotzu', content: 'The flame that burns twice as bright burns half as long.' },
  { username: 'laotzu', content: 'Care about what other people think and you will always be their prisoner.' },
  { username: 'laotzu', content: 'If you are depressed you are living in the past. If you are anxious you are living in the future. If you are at peace you are living in the present.' },
  { username: 'laotzu', content: 'Simplicity, patience, compassion. These three are your greatest treasures.' },
  { username: 'laotzu', content: 'Do the difficult things while they are easy and do the great things while they are small.' },
  { username: 'laotzu', content: 'Being deeply loved by someone gives you strength, while loving someone deeply gives you courage.' },
  { username: 'laotzu', content: 'To the mind that is still, the whole universe surrenders.' },

  // Rumi (1207-1273)
  { username: 'rumi', content: 'The wound is the place where the Light enters you.' },
  { username: 'rumi', content: 'Yesterday I was clever, so I wanted to change the world. Today I am wise, so I am changing myself.' },
  { username: 'rumi', content: 'Don\'t be satisfied with stories, how things have gone with others. Unfold your own myth.' },
  { username: 'rumi', content: 'Let yourself be silently drawn by the strange pull of what you really love. It will not lead you astray.' },
  { username: 'rumi', content: 'The art of knowing is knowing what to ignore.' },
  { username: 'rumi', content: 'Stop acting so small. You are the universe in ecstatic motion.' },
  { username: 'rumi', content: 'Raise your words, not voice. It is rain that grows flowers, not thunder.' },
  { username: 'rumi', content: 'What you seek is seeking you.' },
  { username: 'rumi', content: 'The quieter you become, the more you are able to hear.' },
  { username: 'rumi', content: 'Set your life on fire. Seek those who fan your flames.' },
  { username: 'rumi', content: 'Sell your cleverness and buy bewilderment.' },
  { username: 'rumi', content: 'Let the beauty we love be what we do. There are hundreds of ways to kneel and kiss the ground.' },
];

function seedDatabase() {
  console.log('Starting database seed...');

  // Initialize the database first
  initDatabase();

  // Clear existing data
  console.log('Clearing existing data...');
  db.prepare('DELETE FROM reactions').run();
  db.prepare('DELETE FROM posts').run();

  // Insert quotes
  console.log(`Inserting ${quotes.length} quotes...`);
  const insertStmt = db.prepare('INSERT INTO posts (username, content, created_at) VALUES (?, ?, ?)');

  // Insert quotes in reverse chronological order (oldest first in DB, newest at top of feed)
  const now = new Date();
  quotes.forEach((quote, index) => {
    // Spread posts over the last 30 days randomly
    const daysAgo = Math.floor(Math.random() * 30);
    const hoursAgo = Math.floor(Math.random() * 24);
    const minutesAgo = Math.floor(Math.random() * 60);

    const timestamp = new Date(now);
    timestamp.setDate(timestamp.getDate() - daysAgo);
    timestamp.setHours(timestamp.getHours() - hoursAgo);
    timestamp.setMinutes(timestamp.getMinutes() - minutesAgo);

    insertStmt.run(quote.username, quote.content, timestamp.toISOString());
  });

  console.log(`✅ Successfully seeded database with ${quotes.length} quotes from 20 philosophers and authors!`);
  console.log('\nAuthors included:');
  const uniqueAuthors = [...new Set(quotes.map(q => q.username))];
  uniqueAuthors.forEach(author => {
    const count = quotes.filter(q => q.username === author).length;
    console.log(`  - ${author}: ${count} quotes`);
  });
}

// Run the seed function
seedDatabase();
