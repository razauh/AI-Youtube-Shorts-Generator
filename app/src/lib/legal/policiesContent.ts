export type PolicyTab = 'terms' | 'privacy' | 'refund';

type PolicySection = {
  heading: string;
  paragraphs: string[];
};

export const POLICY_SECTIONS: Record<PolicyTab, PolicySection[]> = {
  terms: [
    {
      heading: 'Terms of Use',
      paragraphs: [
        'By using this software, you confirm you have rights to process input media and comply with platform policies and local laws.',
        'You are responsible for generated outputs, publication decisions, copyright compliance, and any third-party claims.'
      ]
    },
    {
      heading: 'Acceptable Use',
      paragraphs: [
        'Do not use the app to generate unlawful, deceptive, harmful, or infringing content. Abuse may result in access suspension for paid services.'
      ]
    }
  ],
  privacy: [
    {
      heading: 'Privacy Notice',
      paragraphs: [
        'Local workspace data for generation history and library is stored on-device in this release build. Cloud operations only occur when API mode is selected for generation.'
      ]
    }
  ],
  refund: [
    {
      heading: 'Refund Policy',
      paragraphs: [
        'Refund requests are handled manually within 7 days from purchase, subject to purchase records and platform dispute rules.',
        'No automated refund engine is built into this app.'
      ]
    }
  ]
};

export const POLICY_COMMON_SECTIONS: PolicySection[] = [
  {
    heading: 'Warranty and Liability',
    paragraphs: [
      'The software is provided as-is without guarantees of uninterrupted service. Liability is limited to the maximum extent permitted by law.'
    ]
  },
  {
    heading: 'Support and Contact',
    paragraphs: [
      'For enterprise support and data requests, use the Support section and attach generated debug logs.'
    ]
  }
];

export const POLICY_LAST_UPDATED_LABEL = 'May 8, 2026';
